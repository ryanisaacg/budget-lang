module Account (Account, getBalance, makeDeposit) where
    data Inflow = Fixed Rational | Flex Rational deriving (Show)

    -- Leaf Balance | Branch [(Account, Inflow, Maybe Max)]
    data AccountData = Leaf Rational | Branch [(Account, Inflow, Maybe Rational)]

    data Account = Account String AccountData

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

    makeFixedDeposit :: (Account, Inflow, Maybe Rational) -> ([(Account, Inflow, Maybe Rational)], Rational) -> ([(Account, Inflow, Maybe Rational)], Rational)
    makeFixedDeposit (account, (Fixed input), max) (list, remaining) =
        let deposit = amountDeposit account max input remaining in
            (((makeDeposit account deposit, Fixed input, max):list), remaining - deposit)

    makeFlexDeposit :: Rational -> Rational -> (Account, Inflow, Maybe Rational) -> (Account, Inflow, Maybe Rational)
    makeFlexDeposit _ _ (account, Fixed amount, max) = (account, Fixed amount, max)
    makeFlexDeposit totalFlex deposit (account, inflow, max) =
        (makeDeposit account ((freeRemaining (account, inflow, max)) / totalFlex * deposit), inflow, max)

    -- the account, its optional maximum, the amount the account wants, the amount of money left
    amountDeposit :: Account -> Maybe Rational -> Rational -> Rational -> Rational
    amountDeposit _ Nothing wants left = min wants left
    amountDeposit account (Just maximum) wants left
      | getBalance account >= maximum = 0
      | otherwise = min wants left

    getFlex :: (Account, Inflow, Maybe Rational) -> Rational
    getFlex (_, (Flex amount), Nothing) = amount
    getFlex (account, (Flex amount), Just maximum)
      | getBalance account >= maximum = 0
      | otherwise = amount
    freeRemaining _ = 0
