macro_rules! array {
    ( let $name:ident = Array($dims:expr, $data:expr) ) => {
        // ensures that the name of the base is shadowed
        // and won't be used accidentally
        let $name = crate::array::ArrayBase::new_checked($dims, $data).expect("failed to create array");
        let $name = <crate::array::DenseArray<i32> as crate::array::PartialView>::from_base(&$name);
    };
    ( let $name:ident = IDX($filename:expr) ) => {
        let $name = crate::array::ArrayBase::<u8>::from_idx($filename).expect("failed to read idx");
        let $name = <crate::array::DenseArray<u8> as crate::array::PartialView>::from_base(&$name);
    };
    ( let $name:ident = $num:expr ) => {
        array!(let $name = Array(vec![1], vec![$num]));
    };
    ( let $name:ident =? $base:expr ) => {
        let $name = $base.expect("failed to unwrap ArrayBase");
        let $name = <crate::array::DenseArray<i32> as crate::array::PartialView>::from_base(&$name);
    };
}

macro_rules! shape {
    ( let $name:ident = Shape($dims:expr) ) => {
        let $name = crate::array::ShapeBase::new_checked($dims).expect("failed to create shape");
        let $name = <crate::array::Shape as crate::array::PartialView>::from_base(&$name);
    };
    ( let $name:ident = $dim:expr ) => {
        shape!(let $name = Shape(vec![$dim]));
    };
    ( let $name:ident =? $broadcasted:expr ) => {
        let ($name, _) = $broadcasted.expect("broadcast failed");
        let $name = <crate::array::Shape as crate::array::PartialView>::from_base(&$name);
    };
}

mod array_tests {
    use crate::array::MultiDimensional;
    #[test]
    fn test_eq_1() {
        array!(let arr1 = Array(vec![2,2], vec![0,1,2,3]));
        array!(let arr2 = Array(vec![2,2], vec![0,1,2,3]));

        assert_eq!(arr1, arr2)
    }

    #[test]
    fn test_eq_2() {
        array!(let arr1 = 15);
        array!(let arr2 = 15);

        assert_eq!(arr1, arr2)
    }

    #[test]
    fn test_cast1() {
        array!(let arr1 = 10);
        array!(let arr2 = Array(vec![4], vec![0, 1, 2, 3]));

        array!(let expected = Array(vec![4], vec![0, 10, 20, 30]));
        array!(let actual =? arr1.mul_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast2() {
        array!(let arr1 = 10);
        array!(let arr2 = Array(vec![2,2], vec![0,1,2,3]));

        array!(let expected = Array(vec![2,2], vec![0,10,20,30]));
        array!(let actual =? arr1.mul_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast3() {
        array!(let arr1 = Array(vec![2], vec![0, 1]));
        array!(let arr2 = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        array!(let expected = Array(vec![2, 3], vec![0, 1, 0, 3, 0, 5]));

        array!(let actual =? arr1.mul_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast4() {
        array!(let arr1 = Array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]));
        array!(let arr2 = Array(vec![1, 2], vec![0, 1]));
        array!(let expected = Array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]));

        array!(let actual =? arr1.mul_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast5() {
        array!(let arr1 = Array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]));
        array!(let arr2 = Array(vec![1, 2], vec![0, 1]));
        array!(let expected = Array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]));

        array!(let actual =? arr1.mul_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn plus1() {
        array!(let arr1 = Array(vec![2,2], vec![0,1,2,3]));
        array!(let arr2 = Array(vec![2,2], vec![3,2,1,0]));
        array!(let expected = Array(vec![2,2], vec![3,3,3,3]));

        array!(let actual =? arr1.add_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn plus2() {
        array!(let arr1 = 10);
        array!(let arr2 = Array(vec![2,2], vec![12,13,14,15]));
        array!(let expected = Array(vec![2,2], vec![22,23,24,25]));

        array!(let actual =? arr1.add_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn minus1() {
        array!(let arr1 = Array(vec![2,2], vec![0,1,2,3]));
        array!(let arr2 = Array(vec![2,2], vec![3,2,1,0]));
        array!(let expected = Array(vec![2,2], vec![-3,-1,1,3]));

        array!(let actual =? arr1.sub_v(&arr2));

        assert_eq!(expected, actual);
    }

    #[test]
    fn minus2() {
        array!(let arr1 = 10);
        array!(let arr2 = Array(vec![2,2], vec![12,13,14,15]));
        array!(let expected = Array(vec![2,2], vec![-2,-3,-4,-5]));

        array!(let actual =? arr1.sub_v(&arr2));

        assert_eq!(expected, actual);
    }
}

mod shape_tests {
    use crate::array::Error;

    #[test]
    fn test_eq1() {
        shape!(let shape1 = Shape(vec![3,4,5]));
        shape!(let shape2 = Shape(vec![3,4,5]));
        assert_eq!(shape1, shape2);
    }

    #[test]
    fn test_eq2() {
        shape!(let shape1 = Shape(vec![1000, 1000, 1000]));
        shape!(let shape2 = Shape(vec![1000, 1000, 1000]));
        assert_eq!(shape1, shape2);
    }

    #[test]
    fn test_ne1() {
        shape!(let shape1 = Shape(vec![1000, 1000, 1000]));
        shape!(let shape2 = Shape(vec![1001, 1000, 1000]));
        assert_ne!(shape1, shape2);
    }

    #[test]
    fn test_ne2() {
        shape!(let shape1 = Shape(vec![1, 2, 3]));
        shape!(let shape2 = Shape(vec![1, 2]));
        assert_ne!(shape1, shape2);
    }

    #[test]
    fn test_cast_ok1() {
        shape!(let shape1 = Shape(vec![3, 256, 256]));
        shape!(let shape2 = Shape(vec![3]));
        shape!(let actual =? shape1.broadcast(&shape2));
        shape!(let expected = Shape(vec![3, 256, 256]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_ok2() {
        shape!(let shape1 = Shape(vec![1, 6, 1, 8]));
        shape!(let shape2 = Shape(vec![5, 1, 7]));
        shape!(let actual =? shape1.broadcast(&shape2));
        shape!(let expected = Shape(vec![5, 6, 7, 8]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_ok3() {
        shape!(let a = Shape(vec![4, 5]));
        shape!(let b = Shape(vec![1]));
        shape!(let actual =? a.broadcast(&b));
        shape!(let expected = Shape(vec![4, 5]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_ok4() {
        shape!(let a = Shape(vec![4, 5]));
        shape!(let b = Shape(vec![4]));
        shape!(let actual =? a.broadcast(&b));
        shape!(let expected = Shape(vec![4, 5]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_ok5() {
        shape!(let a = Shape(vec![5, 3, 15]));
        shape!(let b = Shape(vec![5, 1, 15]));
        shape!(let actual =? a.broadcast(&b));
        shape!(let expected = Shape(vec![5, 3, 15]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_ok6() {
        shape!(let a = Shape(vec![5, 3, 15]));
        shape!(let b = Shape(vec![5, 3]));
        shape!(let actual =? a.broadcast(&b));
        shape!(let expected = Shape(vec![5, 3, 15]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_ok7() {
        shape!(let a = Shape(vec![5, 3, 15]));
        shape!(let b = Shape(vec![1, 3]));
        shape!(let actual =? a.broadcast(&b));
        shape!(let expected = Shape(vec![5, 3, 15]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast_err1() {
        shape!(let shape1 = Shape(vec![3]));
        shape!(let shape2 = Shape(vec![4]));
        let actual = shape1.broadcast(&shape2);
        let expected = Error::Broadcast {
            dims1: shape1.to_vec(),
            dims2: shape2.to_vec(),
        };
        assert_eq!(expected, actual.unwrap_err());
    }

    #[test]
    fn test_cast_err2() {
        shape!(let shape1 = Shape(vec![1, 2]));
        shape!(let shape2 = Shape(vec![3, 4, 8]));
        let actual = shape1.broadcast(&shape2);
        let expected = Error::Broadcast {
            dims1: shape1.to_vec(),
            dims2: shape2.to_vec(),
        };
        assert_eq!(expected, actual.unwrap_err());
    }

    #[test]
    fn test_volume1() {
        shape!(let a = Shape(vec![3, 256, 256]));
        let expected = 3 * 256 * 256;
        assert_eq!(expected, a.volume());
    }

    #[test]
    fn test_volume2() {
        shape!(let a = Shape((1..11).collect()));
        let expected = (1..11).product::<usize>();
        assert_eq!(expected, a.volume());
    }
}

mod derank_slice_tests {
    use crate::array::Error;

    #[test]
    fn test_derank_0() {
        array!(let arr = Array(vec![2, 2], vec![0, 1, 2, 3]));
        let actual = arr.derank(0).expect("deranking returned an error");
        array!(let expected = Array(vec![2], vec![0, 1]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_derank_1() {
        array!(let arr = Array(vec![2, 2], vec![0, 1, 2, 3]));
        let actual = arr.derank(1).expect("deranking returned an error");
        array!(let expected = Array(vec![2], vec![2, 3]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_derank_err_1d() {
        array!(let arr = Array(vec![2], vec![0, 1]));
        let result = arr.derank(1);
        assert_eq!(result.unwrap_err(), Error::Derank1D)
    }

    #[test]
    fn test_derank_err_invalid_index() {
        array!(let arr = Array(vec![2, 2], vec![0, 1, 2, 3]));
        let result = arr.derank(2);
        assert_eq!(
            result.unwrap_err(),
            Error::DerankIndexOutOfBounds { len: 2, index: 2 }
        )
    }

    #[test]
    fn test_slice_0_to_2() {
        array!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let actual = arr.slice(0, 2).expect("slicing returned an error");
        array!(let expected = Array(vec![2, 2], vec![0, 1, 2, 3]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_slice_1_to_3() {
        array!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let actual = arr.slice(1, 3).expect("slicing returned an error");
        array!(let expected = Array(vec![2, 2], vec![2, 3, 4, 5]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_slice_err_zero_width() {
        array!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let result = arr.slice(1, 1);
        assert_eq!(result.unwrap_err(), Error::SliceZeroWidth { index: 1 })
    }

    #[test]
    fn test_slice_err_stop_before_step() {
        array!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let result = arr.slice(2, 1);
        assert_eq!(
            result.unwrap_err(),
            Error::SliceStopBeforeStart { start: 2, stop: 1 }
        )
    }

    #[test]
    fn test_slice_err_stop_past_end() {
        array!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let result = arr.slice(2, 4);
        assert_eq!(
            result.unwrap_err(),
            Error::SliceStopPastEnd { stop: 4, dim: 3 }
        )
    }

    #[test]
    fn test_slice_deranked_1() {
        array!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let sliced = arr.slice(1, 3).expect("slicing returned an error");
        let deranked = sliced.derank(0).expect("deranking returned an error");
        array!(let expected = Array(vec![2], vec![2, 3]));
        assert_eq!(expected, deranked)
    }
}

mod array_idx_tests {
    use crate::array::MultiDimensional;

    #[test]
    fn read_correct_ndims() {
        array!(let test_imgs = IDX("idx-files/t10k-images-idx3-ubyte"));

        assert_eq!(test_imgs.ndims(), 3)
    }

    #[test]
    fn read_correct_dims() {
        array!(let test_imgs = IDX("idx-files/t10k-images-idx3-ubyte"));
        shape!(let expected = Shape(vec![10000, 28, 28]));

        assert_eq!(&expected, test_imgs.shape())
    }
}
