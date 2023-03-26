pub enum BodyTy {
    Fn()
}

pub struct Block {

}

pub struct Til {
    pub body_type: BodyTy,
    pub blocks: Vec<Block>,
}
