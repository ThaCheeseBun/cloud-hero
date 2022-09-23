# Cloud Hero

Hosting Clone Hero charts online to save precious hard drive space\
This program will only act as a reader / writer for the songcache.bin format.\
All files will be served statically from the webserver.

## Notes
* This tool will **NOT** check for errors within the notes.chart / notes.mid files. Always check if there are any bad songs with Clone Hero before using this.
* It does not get data in the same order as Clone Hero, so some metadata like artist or song name might be different.

## Todo
* a lot
* Reading
    * Proper dates
* Writing / Scanning
    * Proper dates
    * ~~Checksum~~
    * Charts
        * ~~notes.chart~~
        * notes.mid
    * ~~Lyrics~~
        * ~~Find `phrase_start` / `phrase_end` / `lyric` in notes.chart~~
        * ~~Midi parsing is a bit harder~~
    * ~~Duplicate detection~~
    * Bug checking and fixing
* Multithreading
* Preview generation (generate a preview audio file for fast scrubbing)

tldr it's not done, don't ask when.