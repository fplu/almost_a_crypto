#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    PreviousBlockNotFound,
    BlockExistButIsNotInAnyBranch,

    VerifyingGenesisBlock,
    BlockContainsNoTransaction,
    BlockProofOfWorkIsNotDone,
    BlockHashIsInvalid,
    BlockAlreadyExist,
    BlockIndexAreNotContiguous,
    BlockPrevHashDoesNotMatch,

    TransactionWasAlreadyDone,
    TryingToSendMoneyFromUnknowUser,
    TransactionOf0,
    WrongTransactionSignature,
    SenderDoNotHaveEnoughMoney,

    TcpListenerBind,
    TcpStreamConnect,
    TcpFailToSend,

    MiningInterupted,
    FailToGetMerkle,

    NotFound,
    InvalidFormat,
    EndOfBuffer,
}
