# gd-image-to-text

[![](https://img.shields.io/badge/download-here-green)](https://github.com/matcool/gd-image-to-text/releases)

Geometry Dash image to text conversion tool using multiline text objects

<img width="70%" src=https://user-images.githubusercontent.com/26722564/189576216-bddd63d7-4b47-4464-b9f5-1178d16bf330.png />

This tool outputs a .gmd file which you can import using [GDShare](https://github.com/HJfod/GDShare-mod/)

*if you're allergic to command lines you can just drag an image into the exe ;)*

```
Usage:
    gd-image-to-text [OPTIONS] <path>

Args:
    <path>                  Path for the input image

Options:
    -g, --grayscale         Turns the image grayscale, making it only use one text object
    -o, --output <path>     Specify path for output .gmd file, if not specified will
                            open a save file dialog instead
    -s, --size <size>       Size for the image in *characters*, given in "WxH" format,
                            if not specified will use the image's actual width and height.
                            Keep in mind GD characters are about twice as tall as their width
    --scale <scale>         Scale for the text objects, defaults to 0.075
```

# FAQ

## How does this work?

Text objects in gd actually support multiline text, and by using characters of similar widths we can abuse this to create ascii art from an image. this tool creates a text object for every color channel (R, G, B) and uses blending to combine them

## Why can't it be higher resolution?

The batch nodes that render all the characters have a limit of 16384 instances. GD has each z layer in a separate batch node so we can use this to have each text object in a separate z layer. \
It's not possible to have a higher resolution by adding more text objects as any characters past the 16384 will simply not render at all, despite being in a different object \
There's still 4 more z layers i could use, however that would make the code more complicated and i think 3 objects is a nice object count :-)

## Is this better than previous methods such as Geometrize?

In terms of object count definitely, but in many other aspects, no. \
Not only does this require you to use font 10, it's also incredibly laggy when moving the object and the text objects use a considerable amount of memory


## todo:

- higher color quality by using 3 extra z layers
- try other fonts
