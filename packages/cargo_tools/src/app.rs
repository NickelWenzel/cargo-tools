pub mod cargo;
pub mod cargo_make;

use iced_headless::{Subscription, Task};

use crate::{
    app::{
        cargo::{Cargo, CargoMessage},
        cargo_make::{CargoMake, CargoMakeMessage},
    },
    runtime::Runtime,
};

#[derive(Debug, Clone)]
pub enum AppMessage<UiT: Ui> {
    Cargo(CargoMessage<UiT::Cargo>),
    CargoMake(CargoMakeMessage<UiT::CargoMake>),
}

use crate::app::cargo::ui::Ui as CargoUi;
use crate::app::cargo_make::ui::Ui as CargoMakeUi;
use AppMessage as Msg;

pub trait Ui {
    type Cargo: CargoUi;
    type CargoMake: CargoMakeUi;
}

pub struct App<UiT: Ui> {
    pub cargo: Cargo<UiT::Cargo>,
    pub cargo_make: CargoMake<UiT::CargoMake>,
}

impl<UiT: Ui + 'static> App<UiT> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg<UiT>) -> Task<Msg<UiT>> {
        match msg {
            Msg::Cargo(msg) => self.cargo.update::<RT>(msg).map(Msg::<UiT>::Cargo),
            Msg::CargoMake(msg) => self.cargo_make.update::<RT>(msg).map(Msg::<UiT>::CargoMake),
        }
    }

    pub fn subscription<RT: Runtime>(&self) -> Subscription<Msg<UiT>> {
        let cargo = self.cargo.subscription::<RT>().map(Msg::<UiT>::Cargo);
        let cargo_make = self
            .cargo_make
            .subscription::<RT>()
            .map(Msg::<UiT>::CargoMake);

        Subscription::batch([cargo, cargo_make])
    }
}
