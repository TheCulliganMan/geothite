"""Retire only kernel intermediates opened by this native recorder process."""
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile


class RecorderScratch:
    def __init__(self, roots=None):
        self.errors=[]
        self.files={}
        if roots is None:
            roots=[Path(tempfile.gettempdir())]
            if sys.platform=='darwin':
                try:
                    value=subprocess.check_output(['getconf','DARWIN_USER_TEMP_DIR'],text=True,timeout=3).strip()
                    if value:roots.append(Path(value))
                except (OSError,subprocess.SubprocessError) as error:
                    self.errors.append(str(error))
        self.roots={Path(root).resolve() for root in roots}

    def observe(self, process_id):
        """Register regular, user-owned kernel files open in the exact PID."""
        try:
            result=subprocess.run(['lsof','-p',str(process_id),'-Fn'],text=True,capture_output=True,timeout=3)
            if result.returncode not in (0,1):
                raise ValueError(f'Cannot inspect native recorder scratch: {result.stderr.strip()}')
            current=None
            for line in result.stdout.splitlines():
                if line.startswith('p'):
                    current=int(line[1:]) if line[1:].isdigit() else None
                elif line.startswith('n') and current==process_id:
                    self._register(Path(line[1:]))
        except (OSError,ValueError,subprocess.SubprocessError) as error:
            self.errors.append(str(error))

    def _register(self, path):
        if not re.fullmatch(r'instruments[A-Za-z0-9]+\.ktrace',path.name):return
        if path.parent.resolve() not in self.roots or path.is_symlink():return
        try:
            info=path.stat()
        except FileNotFoundError:
            return
        if not stat.S_ISREG(info.st_mode):return
        if hasattr(os,'getuid') and info.st_uid!=os.getuid():return
        path=path.parent.resolve()/path.name
        self.files[str(path)]=dict(path=str(path),device=info.st_dev,inode=info.st_ino,
                                  birthtime=getattr(info,'st_birthtime',None))

    def retire(self, recorder_finished):
        """Remove registered identities only after the recorder has closed."""
        for entry in self.files.values():
            path=Path(entry['path'])
            if not recorder_finished:
                entry['disposition']='recorder_still_running'
                continue
            try:
                info=path.lstat()
                same=(info.st_dev==entry['device'] and info.st_ino==entry['inode']
                      and getattr(info,'st_birthtime',None)==entry['birthtime']
                      and stat.S_ISREG(info.st_mode))
                if not same:
                    entry['disposition']='identity_changed_retained'
                    continue
                entry['bytes']=info.st_size
                path.unlink()
                entry['disposition']='removed'
            except FileNotFoundError:
                entry['disposition']='already_removed'
            except OSError as error:
                entry['disposition']='cleanup_failed'
                self.errors.append(str(error))
        return dict(policy='Exact recorder PID, temporary root and file identity; cleanup after recorder exit.',
                    files=list(self.files.values()),errors=self.errors)
