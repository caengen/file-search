# File search

Lookup a bunch of `needles` (search terms) in some `haystack` (the stuff we're searching through) file in pdf or docx format.

### Pre-reqs

Needle file should be in the format:

`<term>,<other identifier>`

`<term2>,<other identifier2>`

etc...

Other identifier is useful if you're generating this file and needles are not guaranteed to be unique.

Only pdf or docx haystack supported.
### Run

`cargo run <needle path> <haystack path>`
