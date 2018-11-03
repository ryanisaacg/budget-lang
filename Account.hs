module Account (Account, Inflow, addChild, getBalance, getBalanceByName, makeDeposit, newRoot, newLeaf, newParent) where

data Inflow = Fixed Rational | Flex Rational deriving (Show)

-- Leaf Balance | Branch [(Account, Inflow, Maybe Max)]
data AccountData = Leaf Rational | Branch [BranchEntry] deriving (Show)

type BranchEntry = (Account, Inflow, Maybe Rational)

data Account = Account String AccountData deriving (Show)

-- Get the total balance of an account
getBalance :: Account -> Rational
getBalance (Account _ (Leaf balance)) = balance
getBalance (Account _ (Branch children)) = sum (map (\(account, _, _) -> getBalance account) children)

-- Create an account that has the deposit applied
makeDeposit :: Account -> Rational -> Account
makeDeposit (Account name (Leaf balance)) amount = Account name (Leaf (balance + amount))
makeDeposit (Account name (Branch [])) amount = Account name (Branch [])
makeDeposit (Account name (Branch (head:tail))) amount =
    let (accounts, remaining) = foldr makeFixedDeposit ([head], amount) tail in
        let totalFlex = sum (map getFlex accounts) in
            Account name (Branch (map (makeFlexDeposit totalFlex remaining) accounts))

-- Create a new root account
newRoot :: Account
newRoot = Account "root" (Branch [])

-- Create a new leaf account
newLeaf :: String -> Rational -> Account
newLeaf name balance = Account name (Leaf balance)

-- Create a new parent account
newParent :: String -> Account
newParent name = Account name (Branch [])

makeFixedDeposit :: BranchEntry -> ([BranchEntry], Rational) -> ([BranchEntry], Rational)
makeFixedDeposit (account, (Fixed input), max) (list, remaining) =
    let deposit = amountDeposit account max input remaining in
        (((makeDeposit account deposit, Fixed input, max):list), remaining - deposit)

makeFlexDeposit :: Rational -> Rational -> BranchEntry -> BranchEntry
makeFlexDeposit _ _ (account, Fixed amount, max) = (account, Fixed amount, max)
makeFlexDeposit totalFlex deposit (account, inflow, max) =
    (makeDeposit account ((freeRemaining (account, inflow, max)) / totalFlex * deposit), inflow, max)

-- the account, its optional maximum, the amount the account wants, the amount of money left
amountDeposit :: Account -> Maybe Rational -> Rational -> Rational -> Rational
amountDeposit _ Nothing wants left = min wants left
amountDeposit account (Just maximum) wants left
  | getBalance account >= maximum = 0
  | otherwise = min wants left

getFlex :: BranchEntry -> Rational
getFlex (_, (Flex amount), Nothing) = amount
getFlex (account, (Flex amount), Just maximum)
  | getBalance account >= maximum = 0
  | otherwise = amount
freeRemaining _ = 0

getName :: BranchEntry -> String
getName ((Account name _), _, _) = name

insertChild :: [BranchEntry] -> BranchEntry -> [BranchEntry]
insertChild [] entry = [entry]
insertChild (head:tail) entry
  | getName entry < getName head = (head:insertChild tail entry)
  | otherwise = (entry:head:tail)


-- Add a child to a specific named account
addChild :: Account -> String -> Account -> Inflow -> Maybe Rational -> Maybe Account
addChild (Account rootName (Leaf _)) _ _ _ _ = Nothing
addChild (Account rootName (Branch children)) name child inflow max
  | rootName == name = Just (Account rootName (Branch (insertChild children (child, inflow, max))))
  | otherwise = head' (filtermap (\(root, _, _) -> addChild root name child inflow max) children)

-- Get the balance of a given account
getBalanceByName :: Account -> String -> Maybe Rational
getBalanceByName account search = doToAccount (\account -> Just (getBalance account)) account search

-- Withdraw some value from a named account
withdraw :: Account -> String -> Rational -> (Bool, Account)
withdraw root search amount = mapOverAccount (\account ->
    case account of
      Account name (Leaf balance) -> Just (Account name (Leaf (balance - amount)))
      _ -> Nothing) root search

head':: [a] -> Maybe a
head' [] = Nothing
head' (head:_) = Just head

filtermap :: (a -> Maybe b) -> [a] -> [b]
filtermap func aList = foldr (curry (\(next, list) ->
    case func next of
      Just next -> (next:list)
      Nothing -> list)) [] aList

doToAccount :: (Account -> Maybe a) -> Account -> String -> Maybe a
doToAccount func (Account rootName accountData) searchName
  | rootName == searchName = func (Account rootName accountData)
  | otherwise =
      case accountData of
        Leaf _ -> Nothing
        Branch children -> head' (filtermap (\(child, _, _) -> doToAccount func child searchName) children)

mapOverAccount :: (Account -> Maybe Account) -> Account -> String -> (Bool, Account)
mapOverAccount func (Account rootName accountData) searchName
  | rootName == searchName =
      case func (Account rootName accountData) of
        Just account -> (True, account)
        Nothing -> (False, (Account rootName accountData))
  | otherwise =
      case accountData of
        Leaf _ -> (False, (Account rootName accountData))
        Branch children -> let (found, children) = mapIntegrate (\account -> mapOverAccount func account searchName) children in
                               (found, Account rootName (Branch children))

mapIntegrate :: (Account -> (Bool, Account)) -> [BranchEntry] -> (Bool, [BranchEntry])
mapIntegrate func array = foldr (curry (\((justFound, next), (found, list)) -> (justFound || found, next:list))) (False, []) (map (\(account, influx, max) ->
    let (found, account) = func account in
        (found, (account, influx, max))) array)

treefold :: 
