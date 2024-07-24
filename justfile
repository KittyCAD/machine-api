slice-mk3s $FILE:
    prusa-slicer --load mk3.ini --export-gcode $FILE

slice-mk3s-supported $FILE:
    prusa-slicer --load mk3.ini --support-material --export-gcode $FILE
