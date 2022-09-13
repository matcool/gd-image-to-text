# gd-image-to-text

Geometry Dash image to text conversion tool using multiline text objects

![image](https://user-images.githubusercontent.com/26722564/189576216-bddd63d7-4b47-4464-b9f5-1178d16bf330.png)

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

## todo:

- higher color quality by using 3 extra z layers
- try other fonts
