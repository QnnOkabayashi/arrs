// TODO: combine cast tests from array_tests and shape_tests
use crate::array::{DenseArray, Array1, ArrayBase, Shape, Shape1, ShapeBase, TypeAware, PartialView};
use std::sync::Arc;

fn new_array1<T: TypeAware>(shape: Vec<usize>, data: Vec<T>) -> Array1<T> {
    let shape = Shape1::new(shape);
    let data = Arc::new(data);

    Array1::new(shape, data).expect("Data doesn't contain correct number of items for Shape")
}

macro_rules! arrs {
    ( let $name:ident = Array($dims:expr, $data:expr) ) => {
        // ensures that the name of the base is shadowed
        // and won't be used accidentally
        let $name = crate::array::ArrayBase::new_checked($dims, $data).expect("failed to create array");
        let $name = <crate::array::DenseArray<i32> as crate::array::PartialView>::from_base(&$name);
    };
    ( let $name:ident = $num:expr ) => {
        arrs!(let $name = Array(vec![1], vec![$num]));
    };
    ( let $name:ident =? $base:expr ) => {
        let $name = $base.expect("failed to unwrap ArrayBase");
        let $name = <crate::array::DenseArray<i32> as crate::array::PartialView>::from_base(&$name);
    };
}

mod array_tests {
    #[test]
    fn test_eq_1() {
        arrs!(let arr1 = Array(vec![2,2], vec![0,1,2,3]));
        arrs!(let arr2 = Array(vec![2,2], vec![0,1,2,3]));

        assert_eq!(arr1, arr2)
    }

    #[test]
    fn test_eq_2() {
        arrs!(let arr1 = 15);
        arrs!(let arr2 = 15);

        assert_eq!(arr1, arr2)
    }

    #[test]
    fn test_cast1() {
        arrs!(let arr1 = 10);
        arrs!(let arr2 = Array(vec![4], vec![0, 1, 2, 3]));

        arrs!(let expected = Array(vec![4], vec![0, 10, 20, 30]));
        arrs!(let actual =? arr1.mul(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast2() {
        arrs!(let arr1 = 10);
        arrs!(let arr2 = Array(vec![2,2], vec![0,1,2,3]));

        // broken, figure out later
        arrs!(let expected = Array(vec![2,2], vec![0,10,20,30]));
        arrs!(let actual =? arr1.mul(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast3() {
        arrs!(let arr1 = Array(vec![2], vec![0, 1]));
        arrs!(let arr2 = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        arrs!(let expected = Array(vec![2, 3], vec![0, 1, 0, 3, 0, 5]));

        arrs!(let actual =? arr1.mul(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast4() {
        arrs!(let arr1 = Array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]));
        arrs!(let arr2 = Array(vec![1, 2], vec![0, 1]));
        arrs!(let expected = Array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]));

        arrs!(let actual =? arr1.mul(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast5() {
        arrs!(let arr1 = Array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]));
        arrs!(let arr2 = Array(vec![1, 2], vec![0, 1]));
        arrs!(let expected = Array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]));

        arrs!(let actual =? arr1.mul(&arr2));

        assert_eq!(expected, actual);
    }
}

mod array1_tests {
    use super::new_array1;

    #[test]
    fn test_eq1() {
        let arr1 = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        let arr2 = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        assert_eq!(arr1, arr2);
    }

    #[test]
    fn test_cast1() {
        let arr1 = new_array1(vec![1], vec![10]);
        let arr2 = new_array1(vec![4], vec![0, 1, 2, 3]);
        let expected = new_array1(vec![4], vec![0, 10, 20, 30]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast2() {
        let arr1 = new_array1(vec![1], vec![10]);
        let arr2 = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        let expected = new_array1(vec![2, 2], vec![0, 10, 20, 30]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast3() {
        let arr1 = new_array1(vec![2], vec![0, 1]);
        let arr2 = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let expected = new_array1(vec![2, 3], vec![0, 1, 0, 3, 0, 5]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast4() {
        let arr1 = new_array1(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let arr2 = new_array1(vec![1, 2], vec![0, 1]);
        let expected = new_array1(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast5() {
        let arr1 = new_array1(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let arr2 = new_array1(vec![1, 2], vec![0, 1]);
        let expected = new_array1(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }
}

mod serde_arrs_tests {
    use crate::array::Shape1;
    use crate::serde_arrs;

    #[test]
    fn read_correct_ndims() {
        let test_imgs = serde_arrs::from_idx::<u8>("idx-files/t10k-images-idx3-ubyte").unwrap();
        assert_eq!(test_imgs.ndims(), 3)
    }

    #[test]
    fn read_correct_dims() {
        let test_imgs = serde_arrs::from_idx::<u8>("idx-files/t10k-images-idx3-ubyte").unwrap();

        let expected_shape = Shape1::new(vec![10000, 28, 28]);
        assert_eq!(&expected_shape, test_imgs.shape())
    }
}

mod shape1_tests {
    use crate::array::{Error, Shape1};

    #[test]
    fn test_eq1() {
        let a = Shape1::new(vec![3, 4, 5]);
        let b = Shape1::new(vec![3, 4, 5]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_eq2() {
        let a = Shape1::new(vec![1000, 1000, 1000]);
        let b = Shape1::new(vec![1000, 1000, 1000]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne1() {
        let a = Shape1::new(vec![1000, 1000, 1000]);
        let b = Shape1::new(vec![1001, 1000, 1000]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_ne2() {
        let a = Shape1::new(vec![1, 2, 3]);
        let b = Shape1::new(vec![1, 2]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_cast_ok1() {
        let a = Shape1::new(vec![3, 256, 256]);
        let b = Shape1::new(vec![3]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![3, 256, 256]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok2() {
        let a = Shape1::new(vec![1, 6, 1, 8]);
        let b = Shape1::new(vec![5, 1, 7]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![5, 6, 7, 8]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok3() {
        let a = Shape1::new(vec![4, 5]);
        let b = Shape1::new(vec![1]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![4, 5]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok4() {
        let a = Shape1::new(vec![4, 5]);
        let b = Shape1::new(vec![4]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![4, 5]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok5() {
        let a = Shape1::new(vec![5, 3, 15]);
        let b = Shape1::new(vec![5, 1, 15]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok6() {
        let a = Shape1::new(vec![5, 3, 15]);
        let b = Shape1::new(vec![5, 3]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok7() {
        let a = Shape1::new(vec![5, 3, 15]);
        let b = Shape1::new(vec![1, 3]);
        let res = a.cast(&b);
        let expected = Shape1::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_err1() {
        let lhs = Shape1::new(vec![3]);
        let rhs = Shape1::new(vec![4]);
        let res = lhs.cast(&rhs);
        let expected = Error::Cast { lhs, rhs };
        assert_eq!(expected, res.unwrap_err());
    }

    #[test]
    fn test_cast_err2() {
        let lhs = Shape1::new(vec![1, 2]);
        let rhs = Shape1::new(vec![3, 4, 8]);
        let res = lhs.cast(&rhs);
        let expected = Error::Cast { lhs, rhs };
        assert_eq!(expected, res.unwrap_err());
    }

    #[test]
    fn test_volume1() {
        let a = Shape1::new(vec![3, 256, 256]);
        let expected = 3 * 256 * 256;
        assert_eq!(expected, a.volume());
    }

    #[test]
    fn test_volume2() {
        let a = Shape1::new((1..11).collect());
        let expected = (1..11).product::<usize>();
        assert_eq!(expected, a.volume());
    }
}

mod derank_slice_tests {
    use super::new_array1;
    use crate::array::Error;

    #[test]
    fn test_derank_0() {
        let arr = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        let actual = arr.derank(0).expect("deranking returned an error");
        let expected = new_array1(vec![2], vec![0, 1]);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_derank_1() {
        let arr = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        let actual = arr.derank(1).expect("deranking returned an error");
        let expected = new_array1(vec![2], vec![2, 3]);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_derank_err_1d() {
        let arr = new_array1(vec![2], vec![0, 1]);
        let result = arr.derank(1);
        assert_eq!(result.unwrap_err(), Error::Derank1D)
    }

    #[test]
    fn test_derank_err_invalid_index() {
        let arr = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        let result = arr.derank(2);
        assert_eq!(
            result.unwrap_err(),
            Error::DerankIndexOutOfBounds { len: 2, index: 2 }
        )
    }

    #[test]
    fn test_slice_0_to_2() {
        let arr = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let actual = arr.slice(0, 2).expect("slicing returned an error");
        let expected = new_array1(vec![2, 2], vec![0, 1, 2, 3]);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_slice_1_to_3() {
        let arr = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let actual = arr.slice(1, 3).expect("slicing returned an error");
        let expected = new_array1(vec![2, 2], vec![2, 3, 4, 5]);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_slice_err_zero_width() {
        let arr = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let result = arr.slice(1, 1);
        assert_eq!(result.unwrap_err(), Error::SliceZeroWidth { index: 1 })
    }

    #[test]
    fn test_slice_err_stop_before_step() {
        let arr = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let result = arr.slice(2, 1);
        assert_eq!(
            result.unwrap_err(),
            Error::SliceStopBeforeStart { start: 2, stop: 1 }
        )
    }

    #[test]
    fn test_slice_err_stop_past_end() {
        let arr = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let result = arr.slice(2, 4);
        assert_eq!(
            result.unwrap_err(),
            Error::SliceStopPastEnd { stop: 4, dim: 3 }
        )
    }

    #[test]
    fn test_slice_deranked_1() {
        let arr = new_array1(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let sliced = arr.slice(1, 3).expect("slicing returned an error");
        let deranked = sliced.derank(0).expect("deranking returned an error");
        let expected = new_array1(vec![2], vec![2, 3]);
        assert_eq!(expected, deranked)
    }
}
