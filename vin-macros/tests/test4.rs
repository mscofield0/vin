use vin_macros;

#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin_macros::actor]
#[vin_macros::handles(Msg, bounded(size = 1024, silent))]
struct MyActor {
    pub number: u32,
}

#[async_trait::async_trait]
impl vin_core::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) {
        println!("The message is: {:?}", msg);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        
    }
}