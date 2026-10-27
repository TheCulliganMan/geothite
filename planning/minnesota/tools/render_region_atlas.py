#!/usr/bin/env python3
"""Render an authored regional overview with unchanged, locally supplied source art.

This is a condensed atlas, not the 220-chunk gameplay layout or a pack exporter.
All coordinates below are authored overview metatiles. No geographic generator,
copied source-map block arrays, extracted catalog, or random placement is used.
"""

from render_archive import save_render
import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

from render_reference import SourceArt, load_pack


# The overview preserves the world.json destination graph. Bends and returning
# paths are authored independently; these are not rescaled gameplay coordinates.
ROUTES = [
    ("01", [(435,435),(412,435),(412,442),(395,442),(395,425),(373,425),(373,430),(350,430)],
     [[(412,442),(412,453),(385,453),(385,435),(395,435)], [(395,425),(395,414),(406,414)]],
     [[(376,443),(383,443),(386,447),(386,452),(376,452)]], [(399,418,416,422)]),
    ("02", [(350,430),(330,430),(330,439),(304,439),(304,418),(276,418),(276,426),(249,426),(249,420),(220,420)],
     [[(330,439),(330,450),(309,450),(309,439)], [(276,418),(276,407),(292,407),(292,418)], [(249,426),(249,439),(263,439)]],
     [[(282,432),(289,432),(293,436),(293,442),(284,442),(281,439)]], [(236,405,257,410)]),
    ("03", [(220,420),(220,396),(236,396),(236,378),(251,378),(251,362),(270,362),(270,350)],
     [[(220,408),(207,408),(207,388),(220,388),(220,396)], [(236,387),(248,387),(248,399),(236,399),(236,396)]],
     [[(225,370),(230,370),(230,386),(225,391),(223,387),(223,375)]], [(255,376,265,394),(231,353,243,367)]),
    ("04", [(270,350),(294,350),(294,344),(318,344),(318,350),(340,350)],
     [[(291,350),(291,365),(314,365),(314,350),(318,350)]],
     [[(304,335),(308,335),(308,354),(311,358),(311,373),(306,373),(306,360),(304,355)]], []),
    ("05", [(270,350),(270,329),(254,329),(254,316),(236,316),(236,302),(220,302),(220,290)],
     [[(270,329),(281,329),(281,313),(262,313),(262,329)], [(236,307),(246,307),(246,293)]],
     [[(253,302),(257,302),(260,306),(260,310),(253,310)]], [(239,337,255,342)]),
    ("06", [(220,290),(220,270),(228,270),(228,251),(235,251),(235,230)],
     [[(220,280),(208,280),(208,260),(220,260),(220,270)], [(228,251),(214,251),(214,241),(227,241)]],
     [[(235,263),(241,260),(247,263),(247,274),(243,278),(235,278)]], [(242,242,251,253)]),
    ("07", [(235,230),(235,207),(221,207),(221,190),(210,190),(210,175)],
     [[(235,214),(249,214),(249,196),(231,196),(231,207)], [(221,190),(221,180),(233,180)]],
     [[(201,198),(208,195),(214,199),(214,210),(208,215),(200,212)]], [(231,186,244,190)]),
    ("08", [(210,175),(210,151),(199,151),(199,140),(185,140),(185,125),(175,125)],
     [[(210,163),(224,163),(224,143),(210,143),(210,151)], [(199,140),(199,130),(210,130)]],
     [[(182,151),(188,146),(196,150),(198,160),(192,168),(184,168),(180,162)]], []),
    ("09", [(175,125),(196,125),(196,118),(218,118),(218,140),(244,140),(244,159),(263,159),(263,180),(285,180)],
     [[(218,132),(231,132),(231,118),(246,118),(246,140)], [(263,170),(278,170),(278,155),(263,155),(263,159)]],
     [[(227,150),(234,148),(239,153),(239,161),(231,164),(225,159)]], [(199,131,212,137),(246,168,256,184)]),
    ("10", [(285,180),(311,180),(311,190),(335,190),(335,208),(360,208),(360,225)],
     [[(311,180),(311,169),(327,169),(327,180),(311,180)], [(335,198),(346,198),(346,188)]],
     [[(315,201),(322,199),(328,202),(328,212),(321,217),(314,212)]], [(286,191,302,200),(341,176,351,190)]),
    ("11", [(360,225),(373,225),(373,205),(390,205),(390,181),(405,181),(405,160),(423,160),(423,143),(435,143),(435,130)],
     [[(390,194),(378,194),(378,178),(390,178),(390,181)], [(405,169),(415,169),(415,181)], [(423,148),(411,148),(411,136)]],
     [[(397,148),(401,145),(405,148),(405,153),(398,154)]], [(364,191,373,207),(391,157,399,174),(419,117,429,127)]),
    ("12", [(435,130),(435,109),(419,109),(419,94),(401,94),(401,80),(385,80)],
     [[(435,117),(446,117),(446,101),(435,101),(435,109)], [(419,94),(419,82),(429,82)]],
     [[(406,106),(412,102),(417,106),(416,116),(409,120),(403,115)]], [(432,86,443,92)]),
    ("13", [(385,80),(367,80),(367,91),(355,91),(355,110),(340,110)],
     [[(367,91),(379,91),(379,106),(364,106),(364,91),(367,91)], [(355,99),(343,99),(343,89)]],
     [[(350,73),(358,73),(362,76),(361,82),(351,84),(347,80)]], [(330,86,339,99)]),
    ("14", [(175,125),(156,125),(156,144),(137,144),(137,155),(120,155),(120,165)],
     [[(156,133),(144,133),(144,121),(156,121),(156,125)], [(137,144),(127,144),(127,134)]],
     [[(148,151),(153,149),(157,154),(155,162),(147,165),(143,160)]], []),
    ("15", [(235,230),(212,230),(212,239),(191,239),(191,219),(168,219),(168,220),(140,220)],
     [[(212,230),(212,217),(202,217),(202,230),(212,230)], [(168,219),(168,233),(184,233),(184,219)]],
     [[(151,232),(158,229),(165,232),(166,239),(159,244),(150,241)], [(180,203),(186,201),(189,206),(187,211),(179,211)]], []),
    ("16", [(140,220),(115,220),(115,213),(95,213),(95,219),(77,219),(77,210),(60,210)],
     [[(115,220),(115,234),(97,234),(97,219)], [(95,213),(95,201),(110,201)]],
     [[(80,232),(86,229),(91,233),(91,239),(82,241),(77,237)]], []),
    ("17", [(175,125),(175,100),(190,100),(190,85),(210,85),(210,65),(230,65)],
     [[(175,108),(163,108),(163,90),(175,90),(175,100)], [(190,85),(190,72),(199,72)]],
     [[(178,60),(183,58),(188,62),(187,73),(180,77),(176,71)]], [(194,104,206,111)]),
    ("18", [(235,230),(255,230),(255,242),(271,242),(271,265),(285,265)],
     [[(255,242),(245,242),(245,257),(256,257),(256,242),(255,242)]],
     [[(283,230),(293,227),(304,232),(310,240),(310,255),(304,264),(292,261),(285,252),(280,243)]], []),
    ("19", [(120,165),(120,188),(128,188),(128,204),(140,204),(140,220)],
     [[(120,180),(107,180),(107,199),(120,199),(120,188)], [(128,204),(117,204),(117,213)]],
     [[(130,176),(136,173),(141,177),(141,185),(136,191),(129,189)]], []),
    ("20", [(230,65),(254,65),(254,54),(280,54),(280,68),(311,68),(311,57),(338,57),(338,72),(361,72),(361,80),(385,80)],
     [[(280,54),(280,42),(301,42),(301,57),(291,57),(291,68)], [(338,64),(349,64),(349,49),(338,49),(338,57)]],
     [[(249,36),(258,33),(267,36),(270,43),(264,47),(254,46),(247,42)], [(310,83),(320,78),(331,82),(331,91),(321,97),(311,92)]], [(275,77,291,84)]),
]


# Building positions within each hub, deliberately varied street by street.
# h = home, c = clinic, m = mart, g = gym, l = large civic building.
HUBS = {
    "MnWinona": [(3,3,"h"),(11,4,"l"),(3,11,"h"),(13,12,"h"),(3,7,"h"),(8,12,"h")],
    "MnRochester": [(3,3,"c"),(9,3,"h"),(15,3,"h"),(20,3,"h"),(3,9,"m"),(15,9,"g"),(20,9,"h"),(3,14,"h"),(9,14,"h"),(16,14,"l")],
    "MnMankato": [(3,3,"h"),(11,3,"c"),(17,4,"h"),(4,11,"g"),(15,11,"m"),(4,7,"h"),(9,14,"h")],
    "MnMinneapolis": [(3,3,"c"),(9,3,"h"),(15,3,"l"),(23,3,"l"),(3,9,"m"),(9,9,"h"),(22,9,"g"),(27,10,"h"),(3,15,"l"),(11,15,"h"),(19,15,"h"),(26,15,"h"),(3,20,"h"),(10,20,"h"),(18,20,"h"),(26,20,"h")],
    "MnStPaul": [(3,3,"l"),(12,3,"g"),(21,3,"l"),(27,3,"h"),(3,9,"c"),(12,9,"h"),(22,9,"m"),(27,10,"h"),(3,15,"h"),(11,15,"l"),(23,15,"h"),(5,20,"h"),(16,20,"h"),(25,20,"h")],
    "MnStCloud": [(3,3,"c"),(12,4,"h"),(19,3,"h"),(3,12,"h"),(11,11,"m"),(18,11,"g")],
    "MnBrainerd": [(3,3,"c"),(13,3,"l"),(25,4,"h"),(3,12,"m"),(23,12,"g"),(10,18,"h"),(21,19,"h")],
    "MnLeechLake": [(3,4,"c"),(13,3,"h"),(24,3,"h"),(3,15,"h"),(4,9,"m"),(9,18,"l")],
    "MnBemidji": [(3,3,"c"),(15,3,"g"),(3,12,"m"),(16,12,"h"),(4,19,"h"),(15,19,"h")],
    "MnGrandRapids": [(3,3,"m"),(13,3,"c"),(19,10,"h"),(3,13,"g"),(12,19,"h")],
    "MnDuluth": [(3,3,"c"),(10,3,"g"),(18,3,"h"),(3,9,"m"),(11,9,"l"),(20,10,"h"),(4,16,"h"),(11,17,"h"),(17,17,"h")],
    "MnGrandMarais": [(3,3,"c"),(12,3,"h"),(18,3,"l"),(3,11,"m"),(12,11,"h"),(18,11,"h")],
    "MnBoundaryWaters": [(3,3,"l"),(12,3,"h"),(25,4,"h"),(3,17,"h"),(24,17,"c")],
    "MnEly": [(3,3,"c"),(13,3,"l"),(3,12,"m"),(15,12,"h"),(9,19,"g")],
    "MnLakeItasca": [(3,3,"h"),(15,3,"l"),(3,16,"c"),(16,17,"h")],
    "MnDetroitLakes": [(3,3,"c"),(13,3,"m"),(19,8,"h"),(3,12,"h"),(12,12,"g")],
    "MnMoorhead": [(3,3,"h"),(12,3,"c"),(18,3,"g"),(3,11,"m"),(14,12,"h")],
    "MnInternationalFalls": [(3,3,"l"),(15,3,"c"),(3,13,"m"),(15,12,"h"),(10,19,"h")],
    "MnMilleLacs": [(3,3,"c"),(14,3,"h"),(25,4,"h"),(3,14,"m"),(8,19,"h")],
}

# Broad, irregular districts interrupt the narrow traveling lanes. Their
# footprints are intentionally different from the implementation blockouts.
DISTRICTS = {
    "01": [[(361,419),(374,419),(374,412),(389,412),(389,416),(412,416),(412,423),(421,423),(421,438),(415,438),(415,450),(398,450),(398,457),(380,457),(380,446),(365,446)]],
    "02": [[(241,414),(261,414),(261,406),(280,406),(280,411),(309,411),(309,424),(336,424),(336,446),(316,446),(316,455),(297,455),(297,435),(277,435),(277,443),(257,443),(257,434),(241,434)]],
    "03": [[(204,382),(221,382),(221,373),(240,373),(240,361),(258,361),(258,372),(263,372),(263,389),(245,389),(245,406),(228,406),(228,413),(204,413)]],
    "04": [[(286,337),(299,337),(299,333),(313,333),(313,338),(323,338),(323,359),(317,359),(317,370),(290,370)]],
    "05": [[(231,294),(249,294),(249,305),(262,305),(262,309),(284,309),(284,332),(273,332),(273,339),(253,339),(253,324),(232,324)]],
    "06": [[(204,244),(216,244),(216,235),(239,235),(239,247),(248,247),(248,274),(235,274),(235,286),(207,286),(207,270),(200,270),(200,252)]],
    "07": [[(199,188),(213,188),(213,180),(234,180),(234,190),(251,190),(251,216),(239,216),(239,222),(219,222),(219,210),(199,210)]],
    "08": [[(176,136),(193,136),(193,127),(212,127),(212,139),(227,139),(227,165),(216,165),(216,174),(198,174),(198,164),(182,164)]],
    "09": [[(190,111),(222,111),(222,121),(247,121),(247,136),(255,136),(255,153),(277,153),(277,172),(291,172),(291,187),(257,187),(257,170),(235,170),(235,148),(211,148),(211,133),(190,133)]],
    "10": [[(295,163),(320,163),(320,169),(331,169),(331,181),(346,181),(346,199),(366,199),(366,218),(346,218),(346,207),(327,207),(327,196),(302,196),(302,185),(295,185)]],
    "11": [[(363,182),(377,182),(377,173),(390,173),(390,154),(404,154),(404,134),(425,134),(425,149),(414,149),(414,167),(401,167),(401,187),(386,187),(386,208),(372,208),(372,223),(360,223)]],
    "12": [[(396,74),(414,74),(414,83),(431,83),(431,97),(449,97),(449,119),(434,119),(434,125),(417,125),(417,111),(402,111)]],
    "13": [[(341,85),(358,85),(358,74),(383,74),(383,94),(387,94),(387,111),(365,111),(365,119),(348,119),(348,105),(339,105)]],
    "14": [[(126,137),(143,137),(143,119),(161,119),(161,128),(171,128),(171,144),(158,144),(158,160),(139,160),(139,169),(123,169)]],
    "15": [[(150,213),(174,213),(174,204),(194,204),(194,224),(205,224),(205,212),(220,212),(220,226),(225,226),(225,244),(202,244),(202,250),(181,250),(181,237),(162,237),(162,229),(150,229)]],
    "16": [[(71,205),(89,205),(89,197),(113,197),(113,207),(130,207),(130,228),(119,228),(119,239),(95,239),(95,229),(76,229)]],
    "17": [[(162,88),(180,88),(180,78),(196,78),(196,64),(215,64),(215,78),(222,78),(222,90),(203,90),(203,105),(185,105),(185,115),(161,115)]],
    "18": [[(244,230),(265,230),(265,237),(278,237),(278,253),(291,253),(291,272),(267,272),(267,262),(241,262)]],
    "19": [[(103,173),(119,173),(119,166),(137,166),(137,181),(145,181),(145,196),(137,196),(137,212),(119,212),(119,203),(103,203)]],
    "20": [[(244,48),(272,48),(272,37),(303,37),(303,58),(318,58),(318,47),(341,47),(341,60),(353,60),(353,68),(368,68),(368,87),(345,87),(345,77),(321,77),(321,85),(303,85),(303,75),(273,75),(273,63),(244,63)]],
}

# Insets and fingers of habitat are authored against individual landmarks.
# Material 2 is forest, 6 encounter grass, and 7 a native flower bed.
HABITATS = {
    "01": [(6,[(370,419),(383,419),(383,423),(378,423),(378,428),(370,428)]),(2,[(383,432),(391,432),(391,440),(387,440),(387,446),(383,446)]),(6,[(400,426),(410,426),(410,432),(403,432),(403,438),(400,438)]),(2,[(399,445),(407,445),(407,449),(399,449)]),(7,[(414,440),(419,440),(419,443),(414,443)])],
    "02": [(6,[(246,416),(259,416),(259,422),(246,422)]),(6,[(279,409),(289,409),(289,415),(279,415)]),(6,[(310,423),(323,423),(323,429),(310,429)]),(6,[(311,444),(323,444),(323,448),(311,448)]),(2,[(264,423),(271,423),(271,435),(264,435)]),(2,[(294,420),(300,420),(300,431),(294,431)]),(2,[(282,447),(291,447),(291,452),(282,452)])],
    "03": [(2,[(211,390),(217,390),(217,399),(213,399),(213,403),(211,403)]),(6,[(230,380),(233,380),(233,391),(230,391)]),(6,[(245,369),(250,369),(250,375),(245,375)]),(2,[(240,400),(246,400),(246,405),(240,405)])],
    "04": [(2,[(297,353),(301,353),(301,361),(297,361)]),(7,[(316,340),(321,340),(321,342),(316,342)])],
    "05": [(2,[(261,316),(267,316),(267,326),(261,326)]),(6,[(240,299),(248,299),(248,304),(240,304)]),(2,[(241,320),(249,320),(249,328),(241,328)]),(6,[(273,313),(278,313),(278,324),(273,324)])],
    "06": [(2,[(211,263),(217,263),(217,276),(211,276)]),(2,[(218,244),(224,244),(224,250),(218,250)]),(6,[(231,253),(235,253),(235,264),(231,264)]),(6,[(209,249),(214,249),(214,256),(209,256)])],
    "07": [(2,[(239,199),(245,199),(245,210),(239,210)]),(6,[(224,193),(230,193),(230,200),(224,200)]),(2,[(210,202),(216,202),(216,212),(210,212)])],
    "08": [(2,[(214,146),(219,146),(219,159),(214,159)]),(6,[(202,133),(208,133),(208,139),(202,139)]),(2,[(198,153),(204,153),(204,161),(200,161),(200,166),(198,166)])],
    "09": [(2,[(202,122),(211,122),(211,129),(206,129),(206,134),(202,134)]),(6,[(222,137),(228,137),(228,141),(222,141)]),(2,[(249,146),(255,146),(255,154),(249,154)]),(2,[(268,163),(275,163),(275,168),(268,168)]),(6,[(267,175),(278,175),(278,179),(267,179)])],
    "10": [(2,[(314,173),(323,173),(323,178),(314,178)]),(2,[(326,193),(332,193),(332,201),(326,201)]),(6,[(340,203),(349,203),(349,207),(340,207)])],
    "11": [(2,[(381,181),(386,181),(386,189),(381,189)]),(2,[(394,161),(399,161),(399,174),(394,174)]),(6,[(409,154),(417,154),(417,158),(409,158)]),(6,[(415,138),(419,138),(419,147),(415,147)])],
    "12": [(2,[(439,105),(443,105),(443,114),(439,114)]),(2,[(422,98),(429,98),(429,105),(425,105),(425,110),(422,110)]),(6,[(404,84),(412,84),(412,90),(404,90)])],
    "13": [(2,[(369,95),(375,95),(375,103),(369,103)]),(6,[(348,91),(351,91),(351,102),(348,102)]),(2,[(360,79),(365,79),(365,86),(360,86)])],
    "14": [(2,[(148,124),(152,124),(152,131),(148,131)]),(6,[(141,146),(151,146),(151,150),(141,150)]),(2,[(128,148),(133,148),(133,155),(128,155)])],
    "15": [(2,[(204,221),(209,221),(209,228),(204,228)]),(2,[(173,222),(181,222),(181,230),(173,230)]),(6,[(183,242),(197,242),(197,246),(183,246)]),(6,[(153,216),(160,216),(160,220),(153,220)])],
    "16": [(6,[(81,208),(90,208),(90,213),(81,213)]),(6,[(100,204),(110,204),(110,209),(100,209)]),(6,[(101,224),(109,224),(109,230),(101,230)]),(2,[(80,224),(88,224),(88,228),(80,228)])],
    "17": [(2,[(166,94),(171,94),(171,104),(166,104)]),(2,[(193,76),(196,76),(196,81),(202,81),(202,87),(193,87)]),(6,[(178,103),(182,103),(182,112),(178,112)])],
    "18": [(2,[(248,246),(252,246),(252,253),(248,253)]),(6,[(273,255),(279,255),(279,260),(273,260)]),(2,[(260,233),(266,233),(266,238),(260,238)])],
    "19": [(2,[(111,183),(117,183),(117,194),(111,194)]),(6,[(123,172),(128,172),(128,182),(123,182)]),(2,[(131,192),(137,192),(137,199),(131,199)])],
    "20": [(2,[(284,46),(294,46),(294,53),(284,53)]),(2,[(315,61),(325,61),(325,66),(315,66)]),(2,[(342,52),(346,52),(346,61),(342,61)]),(6,[(257,56),(269,56),(269,60),(257,60)]),(6,[(287,71),(298,71),(298,74),(287,74)]),(6,[(347,74),(355,74),(355,78),(347,78)])],
}

SCALE=0.68


def overview_point(point):
    return tuple(round(value*SCALE) for value in point)


class Atlas:
    """Small authored-material canvas; palette fitting happens only at draw time."""

    def __init__(self, width, height):
        self.canvas = Image.new("I", (width,height))
        self.draw = ImageDraw.Draw(self.canvas)
        self.town_art = []
        self.bridges = []

    def line(self, points, material, width):
        self.draw.line(points, fill=material, width=width, joint="curve")

    def region(self, points, material):
        self.draw.polygon(points, fill=material)

    def lake(self, points):
        mask=Image.new("L",self.canvas.size)
        ImageDraw.Draw(mask).polygon(points,fill=255)
        bank=mask.filter(ImageFilter.MaxFilter(7))
        self.canvas.paste(2,(0,0),bank)
        self.region(points,4)

    def route(self, identity, main, loops, lakes, ridges):
        main=[overview_point(p) for p in main]
        loops=[[overview_point(p) for p in loop] for loop in loops]
        lakes=[[overview_point(p) for p in lake] for lake in lakes]
        ridges=[(*overview_point(rect[:2]),*overview_point(rect[2:])) for rect in ridges]
        rocky=identity in {"09","10","11"}
        prairie=identity in {"01","02","16"}
        for path in [main,*loops]:
            self.line(path, 2, 19)
        for district in DISTRICTS[identity]:
            district=[overview_point(p) for p in district]
            mask=Image.new("L",self.canvas.size)
            ImageDraw.Draw(mask).polygon(district,fill=255)
            self.canvas.paste(2,(0,0),mask.filter(ImageFilter.MaxFilter(7)))
            self.region(district,5 if rocky else 1 if prairie else 2)
        for lake in lakes:
            self.lake(lake)
        for ridge in ridges:
            x,y,x1,y1=ridge
            self.draw.rectangle((x-2,y-2,x1+2,y1+2),fill=2)
            self.draw.rectangle(ridge, fill=5)
        for path in [main,*loops]:
            self.line(path, 1, 9 if path is main else 7)
            if not prairie:
                self.line(path, 3, 3 if path is main else 2)
        if prairie:
            self.line(main[:2],3,3)
            self.line(main[-2:],3,3)
        # Native encounter meadows follow the authored bends, with passing bays.
        for index in range(1,len(main)-1,2):
            x,y=main[index]
            self.draw.rectangle((x-5,y-4,x-2,y-1), fill=6)
        for path in loops:
            x,y=path[-2]
            self.draw.rectangle((x+2,y+2,x+4,y+3), fill=7)
        for material,polygon in HABITATS[identity]:
            self.region([overview_point(p) for p in polygon],material)

    def hub(self, town):
        cx,cy=[round(p*5*SCALE) for p in town["reference_anchor_percent"]]
        w,h=town["dimensions_metatiles"]
        x,y=cx-w//2,cy-h//2
        self.draw.rectangle((x-2,y-2,x+w+1,y+h+1),fill=2)
        self.draw.rectangle((x+1,y+1,x+w-2,y+h-2),fill=3)
        self.line([(x,cy),(x+w-1,cy)],3,3)
        self.line([(cx,y),(cx,y+h-1)],3,3)
        # Park space is explicitly kept open at the heart of each settlement.
        self.draw.rectangle((cx-3,cy-3,cx+2,cy+1),fill=1)
        self.draw.rectangle((cx-2,cy-2,cx,cy-1),fill=7)
        self.draw.rectangle((x+1,y+h-5,x+4,y+h-2),fill=2)
        self.draw.rectangle((x+w-6,y+6,x+w-3,y+9),fill=2)
        water={
            "MnWinona":[(17,6),(19,6),(19,16),(18,16),(17,12)],
            "MnLeechLake":[(15,10),(24,8),(29,12),(29,21),(21,22),(15,18)],
            "MnMilleLacs":[(17,10),(24,9),(29,13),(29,21),(20,22),(16,18)],
            "MnBoundaryWaters":[(13,9),(19,7),(23,10),(22,16),(18,19),(12,16)],
            "MnDuluth":[(25,10),(30,7),(30,21),(23,21),(23,17)],
            "MnLakeItasca":[(10,9),(14,8),(17,12),(15,17),(10,17),(8,13)],
        }.get(town["id"])
        if water:self.region([(x+a,y+b) for a,b in water],4)
        for bx,by,kind in HUBS[town["id"]]:
            # Each facade gets a composed lot rather than floating in a broad
            # white clearing. Streets and door approaches remain legible.
            bw=3 if kind=="l" and town["tileset"]!="johto_modern" else 2
            bh=3 if kind=="l" and town["tileset"]=="johto_modern" else 2
            self.draw.rectangle((x+bx-1,y+by-1,x+bx+bw,y+by+bh+1),fill=1)
            self.line([(x+bx, y+by+2),(x+bx,cy)],3,1)
            if kind=="h":self.draw.point((x+bx+bw,y+by+bh),fill=7)
            self.town_art.append((town["tileset"],x+bx,y+by,kind))
        return (x,y,w,h,cx,cy)


def terrain_blocks(canvas):
    w,h=canvas.size
    raw=list(canvas.getdata())
    def at(x,y):
        return raw[y*w+x] if 0<=x<w and 0<=y<h else 0
    blocks=[]
    for y in range(h):
        for x in range(w):
            material=at(x,y)
            block={0:None,1:0x02,2:0x05,3:0x01,4:0x35,5:0x71,6:0x03,7:0x04}[material]
            n,s,west,e=[at(a,b) for a,b in [(x,y-1),(x,y+1),(x-1,y),(x+1,y)]]
            if material==2:
                # Coherent source tree fringe rather than chopped tree crowns.
                open_n=n not in (0,2)
                open_s=s not in (0,2)
                open_w=west not in (0,2)
                open_e=e not in (0,2)
                if open_n:block=0x62
                elif open_s:block=0x5c
                elif open_w:block=0x61
                elif open_e:block=0x60
            elif material==5:
                if n!=5:block=0x70
                if s!=5:block=0x72
                if west!=5:block=0x68
                if e!=5:block=0x69
                if n!=5 and west!=5:block=0x6a
                if n!=5 and e!=5:block=0x6b
                if s!=5 and west!=5:block=0x6c
                if s!=5 and e!=5:block=0x6d
            elif material==4:
                if west!=4:block=0x58 if n==4 else 0x54
                elif e!=4:block=0x59 if n==4 else 0x55
                elif n!=4:block=0x76
            blocks.append(block)
    return blocks


def facade(kind, tileset):
    # Native facade recipes, not sampled rectangles from existing towns.
    if tileset=="kanto":
        if kind=="h":return [[0x38,0x39],[0x3c,0x3d]]
        if kind=="c":return [[0x20,0x21],[0x7c,0x72]]
        if kind=="m":return [[0x20,0x21],[0x7c,0x73]]
        return [[0x0c,0x0d,0x0e],[0x10,0x11,0x12]]
    if tileset=="johto_modern":
        if kind=="g":return [[0x18,0x1f,0x19],[0x1c,0x1d,0x1e]]
        if kind=="l":return [[0x29,0x2a],[0x21,0x22],[0x24,0x28]]
        if kind=="c":return [[0x18,0x19],[0x1a,0x1b]]
        if kind=="m":return [[0x18,0x1f,0x19],[0x27,0x23,0x28],[0x10,0x17,0x33]]
    if kind=="l":return [[0x18,0x1f,0x19],[0x16,0x1c,0x1e]]
    if kind=="g":return [[0x22,0x23],[0x28,0x29]]
    if kind=="c":return [[0x18,0x19],[0x1a,0x1b]]
    if kind=="m":return [[0x18,0x19],[0x1a,0x17]]
    return [[0x18,0x19],[0x16,0x1e]]


def render(pack, root, output):
    world=json.loads((root/"world.json").read_text())
    assert {r["id"] for r in world["routes"]}=={f"MN_R{r[0]}" for r in ROUTES}
    assert {t["id"] for t in world["towns"]}==set(HUBS)
    width,height=round(500*SCALE),round(490*SCALE)
    atlas=Atlas(width,height)
    for identity,main,loops,lakes,ridges in ROUTES:
        atlas.route(identity,main,loops,lakes,ridges)

    # Superior's scalloped northwestern coast follows the authored shore trail.
    # Its open eastern edge is the atlas extent, not an invented land crossing.
    superior=[(374,234),(374,216),(391,216),(391,193),(406,193),
              (406,173),(425,173),(425,155),(444,155),(444,141),
              (465,141),(479,130),(488,130),(488,197),(474,212),
              (451,220),(435,236),(409,250),(389,250)]
    atlas.region([overview_point(p) for p in superior],4)

    # The headwaters and canoe-country lakes are individually drawn footprints.
    for lake in [
        [(104,153),(109,150),(116,153),(115,160),(109,164),(103,161)],
        [(211,161),(220,156),(229,162),(231,172),(226,183),(219,189),(212,185)],
        [(165,119),(170,114),(179,114),(182,120),(180,130),(173,136),(165,131)],
        [(374,70),(380,65),(389,67),(394,75),(392,86),(384,91),(375,88),(370,79)],
        [(350,39),(359,35),(370,38),(372,46),(366,54),(354,55),(349,49)],
        [(401,56),(408,50),(420,52),(425,59),(421,68),(411,73),(403,67)],
        [(139,211),(146,205),(153,209),(155,217),(149,224),(140,222)],
    ]:
        atlas.lake([overview_point(p) for p in lake])
    hub_positions={town["id"]:atlas.hub(town) for town in world["towns"]}
    for x,y,kind in [(405,414,"h"),(292,407,"l"),(263,439,"h"),(227,241,"h"),
                     (233,180,"h"),(210,130,"h"),(246,118,"h"),(327,169,"h"),
                     (411,136,"h"),(429,82,"h"),(343,89,"h"),(127,134,"h"),
                     (202,217,"h"),(110,201,"l"),(199,72,"h"),(117,213,"h"),(301,42,"h")]:
        x,y=overview_point((x,y))
        atlas.town_art.append(("johto",x,y,kind))

    art=SourceArt(pack)
    sets={key:art.blocks(key) for key in ["johto","johto_modern","kanto"]}
    colors=[(19,29,40),(176,194,202),(230,237,233)]
    for blocks in sets.values():
        for block in blocks:
            for color in block.getdata():
                if color not in colors:colors.append(color)
    assert len(colors)<=256
    palette=Image.new("P",(1,1))
    flat=[c for color in colors for c in color]
    palette.putpalette(flat+[0]*(768-len(flat)))
    fitted={key:[b.quantize(palette=palette,dither=Image.Dither.NONE) for b in blocks] for key,blocks in sets.items()}
    image=Image.new("P",(width*32,height*32),0)
    image.putpalette(palette.getpalette())
    for i,block in enumerate(terrain_blocks(atlas.canvas)):
        if block is not None:image.paste(fitted["johto"][block],(i%width*32,i//width*32))
    for key,x,y,kind in atlas.town_art:
        for dy,row in enumerate(facade(kind,key)):
            for dx,block in enumerate(row):
                image.paste(fitted[key][block],((x+dx)*32,(y+dy)*32))
    # Reuse the authored town itself as it evolves in the shared planning
    # folder; never maintain a second exported copy of its block layout.
    marais=root/"playable"/"grand-marais.json"
    if marais.exists():
        town=json.loads(marais.read_text())
        x,y,w,h,cx,cy=hub_positions["MnGrandMarais"]
        x,y=cx-town["width"]//2,cy-town["height"]//2
        rows=[[int(value,16) for value in row.split()] for row in town["rows"]]
        assert len(rows)==town["height"] and all(len(row)==town["width"] for row in rows)
        for dy,row in enumerate(rows):
            for dx,block in enumerate(row):
                image.paste(fitted[town["tileset"]][block],((x+dx)*32,(y+dy)*32))
    # A source-fenced river crossing: water stays visible beside the bridge.
    for x in range(round(301*SCALE),round(311*SCALE)):
        for dy,block in enumerate([0x41,0x01,0x49]):
            image.paste(fitted["johto"][block],(x*32,(round(343*SCALE)+dy)*32))

    output.mkdir(parents=True,exist_ok=True)
    save_render(image, output/"minnesota-region-native.png",optimize=True)
    # Only the overview thumbnail uses area reduction. The native-size source
    # pixels above remain unchanged; integer decimation aliases the tree crowns.
    converted=image.convert("RGB")
    save_render(converted, output/"minnesota-region-native-rgb.png",optimize=True)
    preview=converted.resize((2000,1960),Image.Resampling.BOX)
    save_render(preview, output/"minnesota-region-preview-unlabelled.png")
    draw=ImageDraw.Draw(preview)
    font_path="/System/Library/Fonts/Menlo.ttc"
    title=ImageFont.truetype(font_path,26)
    small=ImageFont.truetype(font_path,11)
    draw.text((40,40),"MINNESOTA",font=title,fill=(230,237,233))
    draw.text((40,78),"19 destinations / 20 routes",font=small,fill=(176,194,202))
    draw.text((40,96),"AUTHORED REGIONAL ATLAS",font=small,fill=(176,194,202))
    draw.text((40,114),"Original source pixels / condensed geography",font=small,fill=(176,194,202))
    for town in world["towns"]:
        x,y,w,h,cx,cy=hub_positions[town["id"]]
        label=town["name"]
        text_width=draw.textlength(label,font=small)
        at=(cx*2000/width-text_width/2,(y+h+2)*1960/height)
        draw.rectangle((at[0]-3,at[1]-1,at[0]+text_width+3,at[1]+13),fill=(19,29,40))
        draw.text(at,label,font=small,fill=(230,237,233))
    draw.text((1700,850),"LAKE SUPERIOR",font=small,fill=(230,237,233))
    draw.text((40,1900),"OVERVIEW ART / Gameplay chunks retain their authored dimensions in world.json.",font=small,fill=(176,194,202))
    save_render(preview, output/"minnesota-region-preview.png")
    print(f"Rendered all 19 hubs and 20 routes: {output}")


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack",type=Path)
    parser.add_argument("--output",type=Path)
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[1]
    render(load_pack(args.pack),root,args.output or root/"renders"/"atlas")
