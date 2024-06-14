Hanayama Puzzle Solutions
=========================

This project is a solutions guide for a subset of the Hanayama cast puzzles that I have solved, namely those that require more complicated solutions.


Solutions Guide
---------------

The main solutions guide is `solutions.tex`.
To build the LaTeX guide, first run `make gfx` to create some figures used in the document.
The run `make` to compile the document as `solutions.pdf`.

Valve Puzzle Model
-----------

For the "Valve" puzzle, a software model was created that let me virtually manipulate the puzzle transparently.
This model was used to derive the solution procedure and provide figures for the guide as to what is going on with the puzzle at each step.
The model is a Rust project in the `valve-model` directory that uses the `easycurses` crate to display and manipulate the model in a command line environment.

Miscellaneous
-------------

Please submit an issue if you discover any errors in the document or have suggestions how to make the guide more clear in any part.
