module TestFunFormat {

    struct SomeOtherStruct has drop {
        some_field: u64,
    }

    public fun multi_arg(/* test comment locate before para */p1:u64,p2:u64):u64{
        p1 + p2
    }
        public fun multi_arg(p1:u64,/* test comment locate before para */p2:u64):u64{
        p1 + p2
    }

    public fun multi_arg(p1/* test comment locate after para */:u64,p2:u64):u64{
        p1 + p2
    }       public fun multi_arg(p1:u64,p2/* test comment locate after para */:u64):u64{
        p1 + p2
    }

        public fun multi_arg(p1:/* test comment locate after para */u64,p2:u64):u64{
        p1 + p2
    }       public fun multi_arg(p1:u64,p2:u64/* test comment locate after para */):u64{
        p1 + p2
    }
}
