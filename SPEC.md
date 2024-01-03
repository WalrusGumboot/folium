# the .flm language specification

## Content, arguments and parameters

By default, folium includes some utilities to get your content laid out quickly. These are largely inspired
by the way Flutter handles declarative UI. 

By **content**, we mean any of the following:

- **centre** takes one single argument of type content and places it in the centre of its bounding box;
- **padding** takes one single argument of type content and adds some padding to it. The amount can be controlled via the `amount` parameter;
- **row** and **column** take at least one argument of type content and lay them out in a row or in a column;
- **text** takes one argument of type string and displays the text; size can be controlled by the `size` parameter (a number representing the size in points), the fill colour can be controlled by the `fill` parameter which is a string with a hex colour code in it)

## Practical presenting
A folium presentation is built up of a sequence of slides.  
A slide is delimited with square brackets (`[`, `]`) and contains a central block of content
and optionally extra styling directives.

The first expression in a slide should be some content to display. By default, a slide renders its content
at full width and full height.

A slide looks roughly like this:
```
[
    // main content
    < element >

    // styling directives for the slide itself
    slide {
        key: value
    }

    // styling directives for named elements 
    element_name {
        key: value
    }
]
```

The `slide` also has some parameters, namely `width`, `height` and `bg`.