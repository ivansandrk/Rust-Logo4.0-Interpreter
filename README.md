# Rust Logo Interpreter

<img src="res/logo.png" alt="PC Logo 4.0" width="640"/>

Interpreter for PC Logo 4.0 written in Rust. My first contact with programming was at the age of 12 when at school we got introduced to Logo and I immediately took a strong liking to it. You write the instructions for the computer, and the computer does them exactly as told. How genius is that!

## Quick start

Clone the repository and run the program with `cargo run`.

## Examples

Load the example programs with (paste in terminal) `LOAD "LOGO\-EXAMPLES\/IVAN.LGO`, and then run one of the following two examples (again paste the full code into terminal):

```
CS PU SETXY [XCOR - 450 YCOR - 100] PD
DIJ 150 20
```

<img src="res/dij.png" alt="DIJ" width="500"/>

```
CS PU SETXY [XCOR - 300 YCOR - 200] PD
MN 500 4 50
```

<img src="res/mn.png" alt="MN" width="500"/>

### Quick glossary

* CS or CLEARSCREEN
* PU or PENUP
* PD or PENDOWN
* SETXY [x y] - sets the coordinates to the list [x y]
* XCOR - value of X coord
* YCOR - value of Y coord
