func main():
    let x = 3
    with_attr error_message("SafeUint256: addition overflow"):
        assert x = 2
    end
    return()
end
