// Read the real PTY's current character screen, not a concatenation of diffs.
// Ratatui can write "SA" and "VE" separately; graphics chunks can also contain
// arbitrary base64. Neither is a reliable substring snapshot of the UI.
export class PtyScreen {
  constructor(columns=80,rows=24) {
    this.x=0;this.y=0;this.saved=[0,0];this.pending='';
    this.resize(columns,rows);
  }
  resize(columns,rows) {
    const old=this.cells??[];
    this.columns=columns;this.rows=rows;
    this.cells=Array.from({length:rows},(_,y)=>Array.from({length:columns},(_,x)=>old[y]?.[x]??' '));
  }
  text() {return this.cells.map(row=>row.join('')).join('\n');}
  feed(text) {
    this.pending+=text;
    let i=0;
    while(i<this.pending.length) {
      const char=String.fromCodePoint(this.pending.codePointAt(i));
      if(char==='\x1b') {
        if(i+1===this.pending.length)break;
        const kind=this.pending[i+1];
        if(kind==='[') {
          const match=/^[0-?]*[ -/]*([@-~])/.exec(this.pending.slice(i+2));
          if(!match)break;
          this.csi(match[0].slice(0,-1),match[1]);i+=2+match[0].length;continue;
        }
        if(kind==='_'||kind===']'||kind==='P') {
          const end=this.pending.indexOf('\x1b\\',i+2);
          const bell=kind===']'?this.pending.indexOf('\x07',i+2):-1;
          if(end<0&&bell<0)break;
          i=bell>=0&&(end<0||bell<end)?bell+1:end+2;continue;
        }
        if(kind==='7')this.saved=[this.x,this.y];
        if(kind==='8')[this.x,this.y]=this.saved;
        i+=2;continue;
      }
      if(char==='\r')this.x=0;
      else if(char==='\n')this.y=Math.min(this.rows-1,this.y+1);
      else if(char==='\b')this.x=Math.max(0,this.x-1);
      else if(char==='\t')this.x=Math.min(this.columns-1,(Math.floor(this.x/8)+1)*8);
      else if(char>=' ') {
        if(this.x>=this.columns){this.x=0;this.y=Math.min(this.rows-1,this.y+1);}
        if(this.cells[this.y])this.cells[this.y][this.x]=char;
        this.x++;
      }
      i+=char.length;
    }
    this.pending=this.pending.slice(i);
  }
  csi(parameters,command) {
    if(parameters.startsWith('?'))return;
    const p=parameters.split(';').map(Number),n=p[0]||1;
    switch(command) {
      case 'H':case 'f':this.y=(p[0]||1)-1;this.x=(p[1]||1)-1;break;
      case 'G':this.x=n-1;break;
      case 'A':this.y=Math.max(0,this.y-n);break;
      case 'B':this.y=Math.min(this.rows-1,this.y+n);break;
      case 'C':this.x=Math.min(this.columns-1,this.x+n);break;
      case 'D':this.x=Math.max(0,this.x-n);break;
      case 'J':if(p[0]===2||p[0]===3)this.cells.forEach(row=>row.fill(' '));break;
      case 'K':this.cells[this.y]?.fill(' ',p[0]===1||p[0]===2?0:this.x,p[0]===1?this.x+1:this.columns);break;
      case 's':this.saved=[this.x,this.y];break;
      case 'u':[this.x,this.y]=this.saved;break;
    }
  }
}
