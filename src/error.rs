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

    NotFound,
    InvalidFormat,
    EndOfBuffer,
}
