#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Nop = 0,
    Exit = 1,
    Call = 2,
    Pcall = 3,
    Ret = 4,
    Throw = 5,
    Jump = 6,
    JumpIf = 7,
    JumpIfNot = 8,

    LoadConst = 9,
    Mov = 10,
    LoadModule = 11,
    LoadFunction = 12,
    LoadGlobal = 13,
    StoreGlobal = 14,
    LoadResource = 15,

    Type = 16,
    ObjectType = 17,

    Add = 18,
    Sub = 19,
    Mul = 20,
    Div = 21,
    Mod = 22,
    Inc = 23,
    Dec = 24,

    Eq = 25,
    Ne = 26,
    ObjectEq = 27,
    ObjectNe = 28,
    Lt = 29,
    Lte = 30,
    Gt = 31,
    Gte = 32,

    And = 33,
    Or = 34,
    Xor = 35,
    Not = 36,

    NewArray = 37,
    ArrayLen = 38,
    ArrayContains = 39,
    ArrayGet = 40,
    ArraySet = 41,
    ArrayInsert = 42,
    ArrayRemove = 43,

    NewMap = 44,
    MapLen = 45,
    MapContains = 46,
    MapGet = 47,
    MapSet = 48,
    MapRemove = 49,

    NewString = 50,
    StringLen = 51,
    StringConcat = 52,
    StringFromChars = 53,
    StringToChars = 54,
    StringFromBytes = 55,
    StringToBytes = 56,
    StringOf = 57,
    StringLowercase = 58,
    StringUppercase = 59,
    StringToNumber = 60,

    NewError = 61,
    ErrorType = 62,
    ErrorMessage = 63,

    NewBuffer = 64,
    BufferLen = 65,
    BufferGet = 66,
    BufferSet = 67,
    NewClosure = 68,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Nop),
            1 => Ok(Self::Exit),
            2 => Ok(Self::Call),
            3 => Ok(Self::Pcall),
            4 => Ok(Self::Ret),
            5 => Ok(Self::Throw),
            6 => Ok(Self::Jump),
            7 => Ok(Self::JumpIf),
            8 => Ok(Self::JumpIfNot),

            9 => Ok(Self::LoadConst),
            10 => Ok(Self::Mov),
            11 => Ok(Self::LoadModule),
            12 => Ok(Self::LoadFunction),
            13 => Ok(Self::LoadGlobal),
            14 => Ok(Self::StoreGlobal),
            15 => Ok(Self::LoadResource),

            16 => Ok(Self::Type),
            17 => Ok(Self::ObjectType),

            18 => Ok(Self::Add),
            19 => Ok(Self::Sub),
            20 => Ok(Self::Mul),
            21 => Ok(Self::Div),
            22 => Ok(Self::Mod),
            23 => Ok(Self::Inc),
            24 => Ok(Self::Dec),

            25 => Ok(Self::Eq),
            26 => Ok(Self::Ne),
            27 => Ok(Self::ObjectEq),
            28 => Ok(Self::ObjectNe),
            29 => Ok(Self::Lt),
            30 => Ok(Self::Lte),
            31 => Ok(Self::Gt),
            32 => Ok(Self::Gte),

            33 => Ok(Self::And),
            34 => Ok(Self::Or),
            35 => Ok(Self::Xor),
            36 => Ok(Self::Not),

            37 => Ok(Self::NewArray),
            38 => Ok(Self::ArrayLen),
            39 => Ok(Self::ArrayContains),
            40 => Ok(Self::ArrayGet),
            41 => Ok(Self::ArraySet),
            42 => Ok(Self::ArrayInsert),
            43 => Ok(Self::ArrayRemove),

            44 => Ok(Self::NewMap),
            45 => Ok(Self::MapLen),
            46 => Ok(Self::MapContains),
            47 => Ok(Self::MapGet),
            48 => Ok(Self::MapSet),
            49 => Ok(Self::MapRemove),

            50 => Ok(Self::NewString),
            51 => Ok(Self::StringLen),
            52 => Ok(Self::StringConcat),
            53 => Ok(Self::StringFromChars),
            54 => Ok(Self::StringToChars),
            55 => Ok(Self::StringFromBytes),
            56 => Ok(Self::StringToBytes),

            57 => Ok(Self::StringOf),
            58 => Ok(Self::StringLowercase),
            59 => Ok(Self::StringUppercase),
            60 => Ok(Self::StringToNumber),

            61 => Ok(Self::NewError),
            62 => Ok(Self::ErrorType),
            63 => Ok(Self::ErrorMessage),

            64 => Ok(Self::NewBuffer),
            65 => Ok(Self::BufferLen),
            66 => Ok(Self::BufferGet),
            67 => Ok(Self::BufferSet),
            68 => Ok(Self::NewClosure),

            _ => Err(()),
        }
    }
}
