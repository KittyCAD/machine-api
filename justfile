slice-mk3s $FILE:
    prusa-slicer --load mk3.ini --rotate-x 180 --export-gcode $FILE

slice-mk3s-supported $FILE:
    prusa-slicer --load mk3.ini --support-material --export-gcode $FILE
