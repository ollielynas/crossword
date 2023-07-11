# open output.txt and raw.txt
import json


output = open("output.txt", "w")
raw = open("raw.txt", "r")
# iter over lines in raw.txt


last = ""
synonyms = []
for i in range(169000):
    line = raw.readline()
    parsed = json.loads(line)
    word = parsed["word"]
    if True in [
            "." in word,
            "-" in word,
            " " in word,
            len(word) <= 3,
            word[0].isupper(),
            word[0].isdigit(),
            word.lower() in [a.lower() for a in parsed["synonyms"]]
            
        ]:continue

    if word == last:
        synonyms += parsed["synonyms"]
    else:
        synonyms = parsed["synonyms"]
        if len(synonyms) > 2:
            output.write(word+"|"+ "|".join(synonyms)+"\n")
    last = word

raw.close()
output.close()