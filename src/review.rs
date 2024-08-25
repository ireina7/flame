#![allow(dead_code)]

use std::fmt;

use anyhow::anyhow;

pub trait Retriever {
    type Id;
    type Item;
    type Err;

    fn get(&self, id: Self::Id) -> Option<&Self::Item>;
    fn retrieve(&mut self) -> Result<Vec<Self::Id>, Self::Err>;
}

pub trait Update {
    type Id;
    type Err;

    fn update(&mut self, id: Self::Id, quality: Quality) -> Result<(), Self::Err>;
}

pub enum Quality {
    CompleteBlackout = 0,
    IncorrectResponse = 1,
    CorrectResponseHard = 2,
    CorrectResponse = 3,
    PerfectResponse = 5,
}

impl Quality {
    pub fn from(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Quality::CompleteBlackout),
            1 => Some(Quality::IncorrectResponse),
            3 => Some(Quality::CorrectResponseHard),
            4 => Some(Quality::CorrectResponse),
            5 => Some(Quality::PerfectResponse),
            _ => None,
        }
    }
}

pub fn logic<App, Err: fmt::Debug>(
    app: &mut App,
    mut qualify: impl FnMut(&<App as Retriever>::Item) -> Result<Option<Quality>, Err>,
) -> Result<bool, Err>
where
    App: Retriever + Update<Id = <App as Retriever>::Id>,
    <App as Retriever>::Id: fmt::Debug + Clone,
    Err: From<<App as Retriever>::Err>,
    Err: From<<App as Update>::Err>,
    Err: From<anyhow::Error>,
{
    let ids = app.retrieve()?;

    for id in ids {
        let item = app
            .get(id.clone())
            .ok_or(anyhow!("failed to get id: {:?}", id.clone()))?;

        let mut quality = qualify(item);
        while let Err(err) = quality {
            eprintln!("[Error] {:?}", err);
            quality = qualify(item);
        }

        let quality = quality.unwrap();
        if let Some(q) = quality {
            app.update(id, q)?;
        } else {
            return Ok(true);
        }
    }

    Ok(false)
}
