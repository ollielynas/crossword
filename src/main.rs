use std::{collections::HashMap, ops::Add, pin::Pin};
use sycamore::{prelude::*, rt::{Event, JsValue}};
use rand::{seq::SliceRandom, Rng};
use web_sys::HtmlDocument;
use std::panic;
use bson::{bson, Bson};
use serde::{Deserialize, Serialize};
use sycamore::rt::JsCast;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize, Copy)]
struct Word {
    text: &'static str,
    pos: Pos,
    orientation: Orientation,
    clue: &'static str,
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

#[derive(Prop)]
struct Props<'a> {
    index: &'a ReadSignal<usize>,
    word: &'a ReadSignal<Word>,
    current_value: &'a ReadSignal<String>,
}


#[component]
fn WordComponent<'a, G: Html>(cx: Scope<'a>, props: Props<'a>) -> View<G> {

        let value = create_signal(cx, "".to_string());

        let disabled = create_memo(cx, move || *props.current_value.get() == props.word.get().text);
        view! { cx,
            input(
                id=props.word.get().text,
                num=props.index.get(),
                placeholder=format!("{}", props.index.get()),
                maxlength=(props.word.get().text).len(),
                disabled=*disabled.get(),
                class=format!(
                "word {} num{}",
                match props.word.get().orientation {
                    Orientation::Horizontal => "horizontal",
                    Orientation::Vertical => "vertical",
                }, props.index.get()),
                style=format!(
                    "left: {}%; top: {}%;width: {}; height: {};",
                    props.word.get().pos.x as f32*20.0/3.0,
                    props.word.get().pos.y as f32*20.0/3.0,
                    match props.word.get().orientation {
                        Orientation::Horizontal => format!("calc({}%)", props.word.get().text.len() as f32*20.0/3.0),
                        Orientation::Vertical => format!("{}%",20.0/3.0),
                    },
                    match props.word.get().orientation {
                        Orientation::Horizontal => format!("{}%",20.0/3.0),
                        Orientation::Vertical => format!("calc({}%)", props.word.get().text.len() as f32*20.0/3.0),
                    },

                ),
                bind:value=value,

            )
        }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Crossword {
    words: Vec<Word>,
    score: i32,
}
#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct CrosswordCookie {
    crossword: Crossword,
    strings: [String;9],
}

impl CrosswordCookie {
    fn load() -> Option<CrosswordCookie> {
        // get html documet
        let window = match web_sys::window() {Some(a) => a, _=> {return None}};
        let document = match window.document() {Some(a) => a, _=> {return None}};
        let html_document:HtmlDocument = match document.dyn_into::<web_sys::HtmlDocument>() {Ok(a) => a, _=> {return None}};
        let string = match html_document.cookie() {Ok(a) => a, _=> {return None}};
    
        let crossword_cookie = match bson::from_bson(bson!{string.replace("x=", "").replace(";", "")}) {Ok(a) => a, _=> {return None}};
        // detect error
        Some(crossword_cookie)
    }

    fn save(&self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let html_document:HtmlDocument = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
        let string = bson::to_bson(&self).unwrap().to_string();
        html_document.set_cookie(&format!("x={};",string)).unwrap();
    }

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

    


    let mut crossword = Crossword::new(&text_file);
    let mut strings = [
        "".to_owned(),
        "".to_owned(),"".to_owned(),
        "".to_owned(),"".to_owned(),
        "".to_owned(),"".to_owned(),
        "".to_owned(),"".to_owned(),];
    if let Some(cookie) = CrosswordCookie::load() {
        cookie.save();
        crossword = cookie.crossword;
        strings = cookie.strings;
    }

    


    sycamore::render(|cx| {


        // i know that this looks bad but it really is the best way to do it
        let words = (0..9).map(|x| create_signal(cx, crossword.words[x].clone())).collect::<Vec<&Signal<Word>>>();
        let values = (0..9).map(|x| create_signal(cx, strings[x].clone())).collect::<Vec<&Signal<String>>>();
        let indexes = (0..9).map(|x| {create_signal(cx, x)}).collect::<Vec<&Signal<usize>>>();

        view! { cx,
            
    ol (type="1") {
        li(class = format!("clue")) {  (words[0].get().clue) }
        li(class = format!("clue")) {  (words[1].get().clue) }
        li(class = format!("clue")) {  (words[2].get().clue) }
        li(class = format!("clue")) {  (words[3].get().clue) }
        li(class = format!("clue")) {  (words[4].get().clue) }
        li(class = format!("clue")) {  (words[5].get().clue) }
        li(class = format!("clue")) {  (words[6].get().clue) }
        li(class = format!("clue")) {  (words[7].get().clue) }
        li(class = format!("clue")) {  (words[8].get().clue) }
    }
        WordComponent(index=indexes[0], word=words[0], current_value=values[0])
        WordComponent(index=indexes[1], word=words[1], current_value=values[1])
        WordComponent(index=indexes[2], word=words[2], current_value=values[2])
        WordComponent(index=indexes[3], word=words[3], current_value=values[3])
        WordComponent(index=indexes[4], word=words[4], current_value=values[4])
        WordComponent(index=indexes[5], word=words[5], current_value=values[5])
        WordComponent(index=indexes[6], word=words[6], current_value=values[6])
        WordComponent(index=indexes[7], word=words[7], current_value=values[7])
        WordComponent(index=indexes[8], word=words[8], current_value=values[8])
            
        
    }});
}