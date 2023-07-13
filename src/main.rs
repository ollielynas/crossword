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
        line = line.replace(|c: char| !" abcdefghijklmnopqrstuvwxyz|:".contains(c), "");
        let mut word_list = line.split("|").collect::<Vec<&str>>().iter().rev().map(|x| *x).collect::<Vec<&str>>();
        let text = word_list.pop().unwrap().to_string();
        word_list.shuffle(&mut rng);
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
            x: rng.gen_range(0..15-self.text.len().min(14)) as i32,
            y: rng.gen_range(0..15-self.text.len().min(14)) as i32,
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
        // its ugly but it works
        
        let text = word.text.clone();
        let ans = text.clone();
        let length = text.clone().len();
        let disabled = create_memo(cx, move || *value.get() == text);
        
        view! { cx,
            input(
                id=ans,
                num=index,
                placeholder=format!("{}", index),
                maxlength=length,
                disabled=*disabled.get(),
                class=format!(
                "word {} num{}",
                match &word.orientation {
                    Orientation::Horizontal => "horizontal",
                    Orientation::Vertical => "vertical",
                }, index),
                style=format!(
                    "left: {}%; top: {}%;width: {}; height: {};",
                    word.pos.x as f32*20.0/3.0,
                    word.pos.y as f32*20.0/3.0,
                    match word.orientation {
                        Orientation::Horizontal => format!("calc({}%)", word.text.len() as f32*20.0/3.0),
                        Orientation::Vertical => format!("{}%",20.0/3.0),
                    },
                    match word.orientation {
                        Orientation::Horizontal => format!("{}%",20.0/3.0),
                        Orientation::Vertical => format!("calc({}%)", word.text.len() as f32*20.0/3.0),
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
            words: vec![],
            score,
        };

        let mut first_word = Word::random_unplaced(&text_file);
        first_word.move_random();
        let mut word_hash = first_word.gen_hash();
        crossword.words.push(first_word);

        for _ in 0..8 {
            let mut free_space_below: HashMap<Pos, i32> = HashMap::new();
            let mut free_space = 0;
            for x in 0..15 {
                for y in (0..15).rev() {
                    free_space += 1;
                    if word_hash.contains_key(&Pos {x,y}) ||word_hash.contains_key(&Pos {x: x+1,y}) ||word_hash.contains_key(&Pos {x: x-1,y}) {
                        free_space = 0;
                    }
                    free_space_below.insert(Pos {x,y}, free_space);
                }
            }
            let mut free_space_right: HashMap<Pos, i32> = HashMap::new();
            let mut free_space = 0;
            for y in 0..15 {
                for x in (0..15).rev() {
                    free_space += 1;
                    if word_hash.contains_key(&Pos {x,y}) ||word_hash.contains_key(&Pos {x,y: y-1}) || word_hash.contains_key(&Pos {x,y: y+1})  {
                        free_space = 0;
                    }
                    free_space_right.insert(Pos {x,y}, free_space);
                }
            }
            let mut valid = false;

            let mut new_word = Word::random_unplaced(&text_file);
            
            while !valid {
                new_word = Word::random_unplaced(&text_file);
                new_word.move_random();

            for (pos, letter) in &word_hash {
                if new_word.text.contains(*letter) {
                    let index = new_word.text.chars().position(|x| &x==letter).unwrap();
                    let top_pos = *pos + match new_word.orientation  {
                        Orientation::Horizontal => Pos {x: -1*index as i32, y: 0},
                        Orientation::Vertical => Pos {x:0,y:-1* index as i32}
                    };
                    let middle_pos = *pos + match new_word.orientation  {
                        Orientation::Horizontal => Pos {x: 1, y: 0},
                        Orientation::Vertical => Pos {x:0,y:1}
                    };

                    if match new_word.orientation {
                        Orientation::Horizontal => &free_space_right,
                        Orientation::Vertical => &free_space_below
                    }.get(&top_pos).unwrap_or(&0) >= &(index as i32) &&
                    match new_word.orientation {
                        Orientation::Horizontal => &free_space_right,
                        Orientation::Vertical => &free_space_below
                    }.get(&middle_pos).unwrap_or(&0) >= &(new_word.text.len() as i32 -  index as i32 -1) 
                    && !word_hash.contains_key(&(top_pos + match new_word.orientation  {
                        Orientation::Horizontal => Pos {x: -1, y: 0},
                        Orientation::Vertical => Pos {x:0,y:-1}
                    }))
                    && !word_hash.contains_key(&(top_pos + match new_word.orientation  {
                        Orientation::Horizontal => Pos {x: new_word.text.len() as i32, y: 0},
                        Orientation::Vertical => Pos {x:0,y:new_word.text.len() as i32}
                    }))
                    && !word_hash.contains_key(&(top_pos + match new_word.orientation  {
                        Orientation::Horizontal => Pos {x: 1, y: 0},
                        Orientation::Vertical => Pos {x:0,y:1}
                    }))
                    && match new_word.orientation  {
                        Orientation::Horizontal =>top_pos.x,
                        Orientation::Vertical => top_pos.y
                    }+ new_word.text.len() as i32 <= 15
                    && !crossword.words.iter().map(|w| w.pos).collect::<Vec<Pos>>().contains(&top_pos)
                    {
                        new_word.move_to(top_pos, new_word.orientation);
                        valid = true;
                        break;
                    }
                }
                }
            }
            for (key, value) in new_word.gen_hash() {
                word_hash.insert(key, value);
            }
            crossword.words.push(new_word);
        }

        return crossword;


    }
}


fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let text_file = include_str!("./output.txt").to_string();



    let crossword = Crossword::new(&text_file);

    sycamore::render(|cx| {
        let words = create_signal(cx, crossword.words.into_iter().enumerate().map(|x| (x.0+1,x.1)).collect::<Vec<(usize, Word)>>());
        let reveal = create_signal(cx, -1);
        view! { cx,
        div(class="just-tell-me-the-answer") {
            (format!("{:#?}", *words.get()))
        }
            ol (type="1") {
            Keyed(
            iterable=words,
            key=|x| x.1.text.clone(),
            view=|cx, x| {
                let num = x.0;
                view! { cx,
                
                li(class = format!("clue{}",x.0)) {  (x.1.clue) }
            }},
        )
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