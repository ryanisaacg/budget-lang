main :: IO ()
main = do
    file <- readFile "Main.hs"
    putStrLn file
