// TODO: Can this struct be made to use references rather than cloning data? I don't think so.
pub struct ClientAttributes{
    pub nick: String,
    pub room: Option<String>
}

impl ClientAttributes{
    pub fn new(nick: String, room: Option<String>) -> Self{
        ClientAttributes { nick, room }
    }
}