# Cloud Hero

Hosting Clone Hero charts online to save precious hard drive space\
This program will only act as a reader / writer for the songcache.bin format.\
All files will be served statically from the webserver.

## Notes
* This tool will **NOT** check for errors within the notes.chart / notes.mid files. Always check if there are any bad songs with Clone Hero before using this.
* It does not get data in the same order as Clone Hero, so some metadata like artist or song name might be different.
* Some midi files might be incompatible with the midi parser (midly). I can't do anything to fix this atm but if you notice any odd looking difficulties, this might be the reason.
* Ini parsing is handled slightly different. Double values will result in the last one being used.

## "Cloud Extended Format"
Trying to find the correct audio files or the right album cover is quite fast on local file storage, but if the game needs to request the server for each one it's going to be real slow. Because of this Cloud Hero includes support for a extended format which includes some more info about the charts. It naturally not backwards compatible and only works with the Cloud Hero client (WIP).

## Todo
* a lot
* Reading
    * Proper dates
* Writing / Scanning
    * Proper dates
    * ~~Checksum~~
    * ~~Charts~~
    * ~~Lyrics~~
    * ~~Duplicate detection~~
    * Bug checking and fixing
* Multithreading
* Preview generation (generate a preview audio file for fast scrubbing)
* CLI interface

tldr it's not done, don't ask when.