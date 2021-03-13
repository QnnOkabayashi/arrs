#[macro_export]
macro_rules! arrs {
    ( let $name:ident = View($base:expr) ) => {
        let $name = crate::array::Array::from_base(&$base);
    };
    ( let $name:ident = Array($dims:expr, $data:expr) ) => {
        let $name = crate::array::ArrayBase::new($dims, $data)?;
        arrs!(let $name = View($name));
    };
    ( let $name:ident = IDX($filename:expr) ) => {
        let $name = crate::array::ArrayBase::<u8, 3>::from_idx($filename)?;
        arrs!(let $name = View($name));
    };

    // using macros for operations because regular functions that are type checked
    // with experimental features crash the compiler due to a bug :(
    ( let $name:ident = add($arr1:expr, $arr2:expr) ) => {
        let $name = $arr1.broadcast_combine($arr2, |a, b| a + b)?;
        arrs!(let $name = View($name));
    };
    ( let $name:ident = sub($arr1:expr, $arr2:expr) ) => {
        let $name = $arr1.broadcast_combine($arr2, |a, b| a - b)?;
        arrs!(let $name = View($name));
    };
    ( let $name:ident = mul($arr1:expr, $arr2:expr) ) => {
        let $name = $arr1.broadcast_combine($arr2, |a, b| a * b)?;
        arrs!(let $name = View($name));
    };
    ( let $name:ident = div($arr1:expr, $arr2:expr) ) => {
        let $name = $arr1.broadcast_combine($arr2, |a, b| a / b)?;
        arrs!(let $name = View($name));
    };
    ( let $name:ident = matmul($arr1:expr, $arr2:expr) ) => {
        let $name = $arr1.matmul($arr2)?;
        arrs!(let $name = View($name));
    };
    ( let $name:ident = $array:expr ) => {
        arrs!(let $name = Array([$array.len()], $array.to_vec()));
    };
}
