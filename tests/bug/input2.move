// first module
module 0xc0ffee::m {
    fun test_return_with_nest() {
        //  test function that not requires line breaks, with many parameters.
        test_func(a,b,c,d,e,f);

        test_func(a,b,c,d,e,f,);

        //  test function that requires line breaks with long name.
        test_func_with_two_long_para(the_first_too_long_long_long_long_parameter,the_second_too_long_long_long_long_parameter);

        //  test function that requires line breaks with long name.many parameters 
        test_func_with_many_para(para1,para2,para3,para4,para2,para3,para1,para2,para3,para1,para2,para3,para1,para2,para3);
    }
}
