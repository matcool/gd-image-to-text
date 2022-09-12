# gd-image-to-text

Geometry Dash image to text conversion tool using multiline text objects

This tool outputs a .gmd file which you can import using [GDShare](https://github.com/HJfod/GDShare-mod/)

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
```

if you're allergic to command lines you can just drag an image into the exe ;)

## todo:

- higher color quality by using 3 extra z layers
- try other fonts
