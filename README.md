## curve_cracker: attempt to crack NIST curve seeds

This is my meager attempt at the challenge here:  https://words.filippo.io/dispatches/seeds-bounty/

Note: requires a wordlist to use for names

## Usage: curve_cracker wordlist logfile(optional)

## Example: curve_cracker names.txt

Matches are printed to console and saved to output.log by default

change MAX_THREADS to match your hardware
change MUT_LEN if you modify number of string mutations
change STRING_VARIATIONS when you change the string templates (that's the whole point, right?)