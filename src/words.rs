use crate::review::{Quality, Retriever, Update};
use anyhow::anyhow;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io};

#[derive(Serialize, Deserialize, Debug)]
pub struct InMemory<Content> {
    version: DateTime<Local>,
    next_id: usize,
    pub mem: HashMap<usize, Content>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item<Payload> {
    repetition: usize,
    factor: f64,
    interval: usize,
    pub payload: Payload,
}

impl<Payload> Item<Payload> {
    pub fn new_interval(&self, n: usize) -> usize {
        match n {
            0 => 0,
            1 => 1,
            2 => 6,
            n => (self.new_interval(n - 1) as f64 * self.factor).ceil() as usize,
        }
    }
}

impl<Payload> InMemory<Item<Payload>> {
    pub fn new(date: DateTime<Local>) -> Self {
        Self {
            version: date,
            next_id: 0,
            mem: HashMap::new(),
        }
    }

    pub fn introduce(&mut self, id: usize, payload: Payload) {
        let item = Item {
            repetition: 0,
            factor: 2.5,
            payload: payload,
            interval: 0,
        };

        self.mem.insert(id, item);
    }

    fn retrieve_ids(&mut self, count_as_a_day: bool) -> anyhow::Result<Vec<usize>> {
        let mut ans = vec![];
        for (id, item) in &mut self.mem {
            if item.interval == 0 {
                ans.push(*id);
            } else if count_as_a_day {
                item.interval -= 1;
            }
        }
        Ok(ans)
    }
}

impl<Payload> Retriever for InMemory<Item<Payload>> {
    type Id = usize;
    type Item = Item<Payload>;
    type Err = anyhow::Error;

    fn get(&self, id: usize) -> Option<&Item<Payload>> {
        return self.mem.get(&id);
    }

    fn retrieve(&mut self) -> Result<Vec<usize>, Self::Err> {
        self.retrieve_ids(true)
    }
}

impl<Payload> Update for InMemory<Item<Payload>> {
    type Id = usize;
    type Err = anyhow::Error;

    fn update(&mut self, id: usize, quality: Quality) -> anyhow::Result<()> {
        let item = self
            .mem
            .get_mut(&id)
            .ok_or(anyhow!("failed to get id: {id:?}"))?;

        item.repetition += 1;

        let q = quality as u8 as f64;
        let mut ef = item.factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
        if ef < 1.3 {
            ef = 1.3
        }

        if ef < 3.0 {
            item.repetition = 0;
        } else {
            item.repetition += 1;
        }
        item.factor = ef;
        item.interval = item.new_interval(item.repetition);

        self.version = Local::now();
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Word {
    pub word: String,
    pub detail: String,
}

impl Word {
    pub fn detail(&self) -> io::Result<String> {
        fs::read_to_string(&self.detail)
    }
}

impl InMemory<Item<Word>> {
    pub fn from(path: &str) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        let db = serde_json::from_reader::<_, Self>(file)?;
        Ok(db)
    }

    pub fn save(self, path: &str) -> io::Result<()> {
        let file = fs::File::create(path)?;
        let ans = serde_json::to_writer(file, &self)?;
        Ok(ans)
    }
}

pub struct App {
    pub count_as_a_day: bool,
    pub db: InMemory<Item<Word>>,
}

impl App {
    pub fn from(path: &str, count_as_a_day: bool) -> io::Result<Self> {
        let app = Self {
            count_as_a_day,
            db: InMemory::from(path)?,
        };
        Ok(app)
    }

    pub fn save(self, path: &str) -> io::Result<()> {
        self.db.save(path)
    }

    fn new_item(word: Word) -> Item<Word> {
        Item {
            repetition: 0,
            factor: 2.5,
            interval: 0,
            payload: word,
        }
    }

    pub fn add(&mut self, word: Word) {
        self.db.mem.insert(self.db.next_id, Self::new_item(word));
        self.db.next_id += 1;
    }
}

impl Retriever for App {
    type Id = usize;
    type Item = Item<Word>;
    type Err = anyhow::Error;

    fn get(&self, id: Self::Id) -> Option<&Self::Item> {
        self.db.get(id)
    }

    fn retrieve(&mut self) -> Result<Vec<Self::Id>, Self::Err> {
        self.db.retrieve_ids(self.count_as_a_day)
    }
}

impl Update for App {
    type Id = usize;
    type Err = anyhow::Error;

    fn update(&mut self, id: Self::Id, quality: Quality) -> Result<(), Self::Err> {
        self.db.update(id, quality)
    }
}
