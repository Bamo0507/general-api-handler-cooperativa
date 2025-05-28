/*
* Function for returning n number of any type value, having a function as a generator
*
* (Ik syntaxis looks scary in the parameters, but it ain't)
*/
pub fn return_n_dummies<T>(dummy_generator: &dyn Fn() -> T, n: i32) -> Vec<T> {
    let mut dummy_list: Vec<T> = vec![];

    //pretty simple logic
    for _ in 0..n {
        dummy_list.push(dummy_generator());
    }

    return dummy_list;
}
