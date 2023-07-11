
use std::{collections::HashMap, ops::Add};

use sycamore::{prelude::*, rt::Event};
use rand::{seq::SliceRandom, Rng};

use std::panic;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct Pos {
    x: i32,
    y: i32,
}
// impl add
impl Add for Pos {
    type Output = Pos;

    fn add(self, other: Pos) -> Pos {
        Pos {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Word {
    text: String,
    pos: Pos,
    orientation: Orientation,
    clue: String,
}

impl Word {
    fn random_unplaced(text_file: &String) -> Word {
        let mut rng = rand::thread_rng();
        let words = text_file.split("\n").collect::<Vec<&str>>();
        let mut line = words.choose(&mut rng).unwrap().to_string();
        line=line.to_lowercase();
        line = line.replace(|c: char| !"abcdefghijklmnopqrstuvwxyz|".contains(c), "");
        let mut word_list = line.split("|").collect::<Vec<&str>>().iter().rev().map(|x| *x).collect::<Vec<&str>>();

        word_list.shuffle(&mut rng);
        let text = word_list.pop().unwrap().to_string();
        let clue = word_list.iter().take(3).map(|x| *x).collect::<Vec<&str>>().join(", ");
        
        

        Word {
            text,
            pos: Pos { x: 0, y: 0 },
            orientation: Orientation::Horizontal,
            clue,
        }
    }

    fn gen_hash(&self) -> HashMap<Pos, char> {
        let mut map = HashMap::new();
        for (i, c) in self.text.chars().enumerate() {
            match self.orientation {
                Orientation::Vertical => map.insert(Pos { x: 0, y: i as i32 } + self.pos, c),
                Orientation::Horizontal => map.insert(Pos { x: i as i32, y: 0 } + self.pos, c),
            };
        }
        return map;
    } 

    fn move_random(&mut self) {
        let mut rng = rand::thread_rng();
        let orientation = [Orientation::Horizontal,Orientation::Vertical][rng.gen_range(0..2)];
        
        self.move_to(Pos {
            x: rng.gen_range(0..15-self.text.len()) as i32,
            y: rng.gen_range(0..15-self.text.len()) as i32,
        }, orientation);
    }

    fn move_to(&mut self, pos: Pos, orientation: Orientation) {
        self.pos = pos;
        self.orientation = orientation;
    }
}

#[component]
    fn WordComponent<G: Html>(cx: Scope, input:(usize, Word)) -> View<G> {
        let (index, word) = input;
        let value = create_signal(cx, String::new());
        let text = word.text.clone();
        let length = text.clone().len();
        let disabled = create_memo(cx, move || *value.get() == text);
        view! { cx,
            input(
                placeholder=format!("{}", index),
                maxlength=length,
                disabled=*disabled.get(),
                class=format!(
                "word {}",
                match &word.orientation {
                    Orientation::Horizontal => "horizontal",
                    Orientation::Vertical => "vertical",
                }),
                style=format!(
                    "left: {}%; top: {}%;width: {}%; height: {}%;",
                    word.pos.x as f32*20.0/3.0,
                    word.pos.y as f32*20.0/3.0,
                    match word.orientation {
                        Orientation::Horizontal => word.text.len() as f32*20.0/3.0,
                        Orientation::Vertical => 20.0/3.0,
                    },
                    match word.orientation {
                        Orientation::Horizontal => 20.0/3.0,
                        Orientation::Vertical => word.text.len() as f32*20.0/3.0,
                    },
                ),
                bind:value=value,

            )
        }
}

struct Crossword {
    words: Vec<Word>,
    score: i32,
}

impl Crossword {
    fn new(text_file: &String) -> Crossword {
        let mut score = 0;
        let mut crossword = Crossword {
            words: (0..6).map(|_| Word::random_unplaced(&text_file)).collect(),
            score,
        };

        let mut hashmaps = crossword.words.iter().map(|x| x.gen_hash()).collect::<Vec<HashMap<Pos, char>>>();
        for (index, word) in crossword.words.iter_mut().enumerate() {
            let mut fail = 0;
            let mut invalid_count = 0;
            loop {
                let mut invalid = false;
                word.move_random();
                let new_hash = word.gen_hash();
                if hashmaps.len() <= index {
                    hashmaps.push(new_hash.clone());
                } else {
                    hashmaps[index] = new_hash.clone();
                }
                let mut overlap = 0;
                for pos in new_hash.keys() {
                    for (i, h) in hashmaps.iter().enumerate() {
                        if i != index {
                            if h.contains_key(pos) {
                                if h.get(pos).unwrap() == new_hash.get(pos).unwrap() {
                                    overlap += 1;
                                    score += 1;
                                }
                                if h.get(pos).unwrap() != new_hash.get(pos).unwrap() {
                                    invalid = true;
                                    invalid_count += 1;
                                }
                            }
                        }
                    }
                }
                if !invalid {
                    if overlap == 0 {
                        fail += 1;
                    }
                    if fail > 100 || overlap > 0 {
                        break;

                    }
                }
                if invalid_count > 1000 {
                    return Crossword::new(&text_file);
                }
            }
        }
        crossword.score = score;


        return crossword;

    }
}


fn main() {

    let text_file = include_str!("./output.txt").to_string();



    let crossword = Crossword::new(&text_file);
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    sycamore::render(|cx| {
        let words = create_signal(cx, crossword.words.into_iter().enumerate().map(|x| (x.0+1,x.1)).collect::<Vec<(usize, Word)>>());
        
        view! { cx,
        div(class="just-tell-me-the-answer") {
            (format!("{:#?}", *words.get()))
        }
        div {
            ol (type="1") {
            Keyed(
            iterable=words,
            view=|cx, x| view! { cx,
                li {  (x.1.clue) }
            },
            key=|x| x.1.text.clone(),
            
        )
    }
        }
        div(class="inputs") {
            Keyed(
            iterable=words,
            view=|cx, x| view! { cx,
                WordComponent(x)
            },
            key=|x| x.1.text.clone(),
        )
        }
    }});
}