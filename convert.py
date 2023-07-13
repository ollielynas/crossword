# open output.txt and raw.txt
import json
import msvcrt
import random

output = open("output.txt", "w")
raw = open("raw.txt", "r")
# iter over lines in raw.txt

common = open("common.txt", "r")
common_words = [c.replace("\n", "") for c in common.readlines()]

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
            word not in common_words,
            len(word) > 10,
            len(word) <= 3,
            word[0].isupper(),
            word[0].isdigit(),
            word.lower() in [a.lower() for a in parsed["synonyms"]]
            
        ]:continue
    
    if word == last:
        synonyms += parsed["synonyms"]
    else:
        synonyms = parsed["synonyms"]
        if len(synonyms) > 3:
            output.write(word+"|"+ "|".join(synonyms)+"\n")
        
    last = word

for word in common_words[100:500]:
    letters = list(word)
    if len(letters) <= 4:
        continue
    random.shuffle(letters)
    output.write(word+"|"+"anagram of: "+"".join(letters)+"\n")

raw.close()
output.close()
common.close()