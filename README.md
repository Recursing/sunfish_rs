# sunfish_rs

 Sunfish is a simple, but strong chess engine, written in Python, mostly for teaching purposes. Without tables and its simple interface, it takes up just 111 lines of code! Yet it plays at rating 1800-1900 at Lichess. It’s simple open source chess engine under the GPL written by Thomas Dybdahl Ahle in Python for didactic purposes, inspired by Harm Geert Muller's Micro- Max. Sunfish supports the Chess Engine Communication protocol to play with a graphical interface like XBoard or PyChess. 
 The program uses Unicode glyphs to display the pieces within the terminal,making it look a little more chess- like than GNU chess. Moves are inputted by specifying the starting and ending co-ordinates,so the aggressive opening which moves the king's pawn to e4 would be inputted e2e4.Note that this can be slightly longer than the more common algebraic notation(in which the previous move would be written e4),but makes it much easier for computation. In this engine Lower case letters represent black pieces (p,r,n,b,k,q), and upper case letters represent white pieces (P,R,N,B,K,Q).Black-White-Empty matrix(BWE) is solely a listing of strings which represent each square on the board. This matrix is compared with an internally stored matrix within the Chess Engine.This suggests the Chess Engine can understand : 
 
i)where the piece moved from 
 
ii)where it moved to and construct a chess command from it. Moving pawn piece A2 to A4 at the beginning of the game would require command ‘a2a4’.         
Rust "port" of the sunfish simple chess engine

# Credits:
- [The original sunfish](https://github.com/thomasahle/sunfish)
- [yuri91](https://github.com/yuri91) for making a first rust port that inspired this

# How to play:
Right now it's only supporting a very basic subset of UCI, will soon add option to choose a simple cli

Challenge it on [lichess](https://lichess.org/@/sunfish_rs) and tell me what you think!

# TODO:
- Improve time managment, maybe rewriting search to be iterative instead of recursive
- Support endgame values (ideally tapered eval), might be tricky to do with incremental updates
- Way more tests, need to test way more positions and to test single functions
- Benchmarking, maybe build a micro benchmarking framework? See https://github.com/bheisler/criterion.rs/issues/306
- Make Square enums more compact, currently they use twice as much memory as the python chars :/
