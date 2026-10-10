#!/usr/bin/env python3
"""Review images preserve smooth corner lighting and intentional flat faces."""
import importlib.util,unittest
from pathlib import Path
import numpy as np
spec=importlib.util.spec_from_file_location('model_review',Path(__file__).with_name('render-model-review.py'))
review=importlib.util.module_from_spec(spec);spec.loader.exec_module(review)

class ReviewNormalTests(unittest.TestCase):
    def test_corner_normals_are_interpolated_and_flat_panels_stay_flat(self):
        p={'positions':[-1.,-1.,0.,1.,-1.,0.,0.,1.,0.],'indices':[0,1,2],'base_color':[.3,.3,.3,1.],'normals':[0.,0.,1.]*3}
        def colors(part):
            rgb=np.asarray(review.render([part],size=160,yaw=0,pitch=0));background=rgb[0,0]
            return np.unique(rgb[np.any(rgb!=background,axis=2)],axis=0)
        self.assertEqual(len(colors(p)),1)
        p['normals']=[-.8,0.,.6,.8,0.,.6,0.,.8,.6]
        self.assertGreater(len(colors(p)),50)
if __name__=='__main__':unittest.main()
