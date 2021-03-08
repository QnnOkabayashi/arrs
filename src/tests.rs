mod array_tests {
    use crate::array::ArrResult;

    #[test]
    fn eq1() -> ArrResult<()> {
        arrs!(let arr1 = Array([2,2], [0,1,2,3]));
        arrs!(let arr2 = Array([2,2], [0,1,2,3]));

        Ok(assert_eq!(arr1, arr2))
    }

    #[test]
    fn eq2() -> ArrResult<()> {
        arrs!(let arr1 = [15]);
        arrs!(let arr2 = Array([1], [15]));

        Ok(assert_eq!(arr1, arr2))
    }

    #[test]
    fn eq3() -> ArrResult<()> {
        arrs!(let arr1 = [1,2,3]);
        arrs!(let arr2 = Array([3], [1,2,3]));

        Ok(assert_eq!(arr1, arr2))
    }

    #[test]
    fn add1() -> ArrResult<()> {
        arrs!(let arr1 = [10]);
        arrs!(let arr2 = Array([2,2], [3,2,1,0]));

        arrs!(let expected = Array([2,2], [13, 12, 11, 10]));
        arrs!(let actual = add(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn mul1() -> ArrResult<()> {
        arrs!(let arr1 = [10]);
        arrs!(let arr2 = Array([4], [0, 1, 2, 3]));

        arrs!(let expected = Array([4], [0, 10, 20, 30]));
        arrs!(let actual = mul(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn mul2() -> ArrResult<()> {
        arrs!(let arr1 = [10]);
        arrs!(let arr2 = Array([2,2], [0,1,2,3]));

        arrs!(let expected = Array([2,2], [0,10,20,30]));
        arrs!(let actual = mul(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn mul3() -> ArrResult<()> {
        arrs!(let arr1 = Array([2], [0, 1]));
        arrs!(let arr2 = Array([2, 3], [0, 1, 2, 3, 4, 5]));

        arrs!(let expected = Array([2, 3], [0, 1, 0, 3, 0, 5]));
        arrs!(let actual = mul(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn mul4() -> ArrResult<()> {
        arrs!(let arr1 = Array([2, 2, 2], [0, 1, 2, 3, 4, 5, 6, 7]));
        arrs!(let arr2 = Array([1, 2], [0, 1]));

        arrs!(let expected = Array([2, 2, 2], [0, 0, 2, 3, 0, 0, 6, 7]));
        arrs!(let actual = mul(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn mul6() -> ArrResult<()> {
        arrs!(let arr1 = Array([2, 2, 2], [0, 1, 2, 3, 4, 5, 6, 7]));
        arrs!(let arr2 = Array([1, 2], [0, 1]));

        arrs!(let expected = Array([2, 2, 2], [0, 0, 2, 3, 0, 0, 6, 7]));
        arrs!(let actual = mul(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn add2() -> ArrResult<()> {
        arrs!(let arr1 = Array([2,2], [0,1,2,3]));
        arrs!(let arr2 = Array([2,2], [3,2,1,0]));

        arrs!(let expected = Array([2,2], [3,3,3,3]));
        arrs!(let actual = add(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn add3() -> ArrResult<()> {
        arrs!(let arr1 = [10]);
        arrs!(let arr2 = Array([2,2], [12,13,14,15]));

        arrs!(let expected = Array([2,2], [22,23,24,25]));
        arrs!(let actual = add(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn sub1() -> ArrResult<()> {
        arrs!(let arr1 = Array([2,2], [0,1,2,3]));
        arrs!(let arr2 = Array([2,2], [3,2,1,0]));

        arrs!(let expected = Array([2,2], [-3,-1,1,3]));
        arrs!(let actual = sub(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn sub2() -> ArrResult<()> {
        arrs!(let arr1 = [10]);
        arrs!(let arr2 = Array([2,2], [12,13,14,15]));

        arrs!(let expected = Array([2,2], [-2,-3,-4,-5]));
        arrs!(let actual = sub(&arr1, &arr2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn vec_dot_vec() -> ArrResult<()> {
        arrs!(let vec1 = [3,4,5]);
        arrs!(let vec2 = [1,0,1]);

        arrs!(let expected = [8]);
        arrs!(let actual = matmul(&vec1, &vec2));

        Ok(assert_eq!(expected, actual))
    }

    #[test]
    fn mat_mul_vec() -> ArrResult<()> {
        arrs!(let mat1 = Array([3,3], [1,2,1,2,3,2,1,2,1]));
        arrs!(let vec1 = [4,4,4]);

        arrs!(let expected = [16,28,16]);
        arrs!(let actual = matmul(&mat1, &vec1));

        Ok(assert_eq!(expected, actual))
    }
}

// mod shape_tests {
//     use crate::array::{ArrResult, Error, Shape};

//     #[test]
//     fn test_eq1() -> ArrResult<()> {
//         let a = Shape::new([3, 4, 5])?;
//         let b = Shape::new([3, 4, 5])?;

//         Ok(assert_eq!(a, b))
//     }

//     #[test]
//     fn test_eq2() -> ArrResult<()> {
//         let a = Shape::new([1000, 1000, 1000])?;
//         let b = Shape::new([1000, 1000, 1000])?;

//         Ok(assert_eq!(a, b))
//     }

//     #[test]
//     fn test_ne1() -> ArrResult<()> {
//         let a = Shape::new([1000, 1000, 1000])?;
//         let b = Shape::new([1001, 1000, 1000])?;

//         Ok(assert_ne!(a, b))
//     }

//     #[test]
//     fn test_ne2() -> ArrResult<()> {
//         let a = Shape::new([1, 2, 3])?;
//         let b = Shape::new([1, 2, 4])?;

//         Ok(assert_ne!(a, b))
//     }

//     #[test]
//     fn test_cast_ok1() -> ArrResult<()> {
//         let a = Shape::new([3, 256, 256])?;
//         let b = Shape::new([3])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([3, 256, 256])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_ok2() -> ArrResult<()> {
//         let a = Shape::new([1, 6, 1, 8])?;
//         let b = Shape::new([5, 1, 7])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([5, 6, 7, 8])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_ok3() -> ArrResult<()> {
//         let a = Shape::new([4, 5])?;
//         let b = Shape::new([1])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([4, 5])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_ok4() -> ArrResult<()> {
//         let a = Shape::new([4, 5])?;
//         let b = Shape::new([4])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([4, 5])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_ok5() -> ArrResult<()> {
//         let a = Shape::new([5, 3, 15])?;
//         let b = Shape::new([5, 1, 15])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([5, 3, 15])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_ok6() -> ArrResult<()> {
//         let a = Shape::new([5, 3, 15])?;
//         let b = Shape::new([5, 3])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([5, 3, 15])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_ok7() -> ArrResult<()> {
//         let a = Shape::new([5, 3, 15])?;
//         let b = Shape::new([1, 3])?;

//         let actual = a.broadcast(&b).unwrap().0;
//         let expected = Shape::new([5, 3, 15])?;

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_err1() -> ArrResult<()> {
//         let a = Shape::new([3])?;
//         let b = Shape::new([4])?;

//         let actual = a.broadcast(&b).unwrap_err();
//         let expected = Error::Broadcast {
//             dims1: a.to_vec(),
//             dims2: b.to_vec(),
//         };

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_cast_err2() -> ArrResult<()> {
//         let a = Shape::new([1, 2])?;
//         let b = Shape::new([3, 4, 8])?;

//         let actual = a.broadcast(&b).unwrap_err();
//         let expected = Error::Broadcast {
//             dims1: a.to_vec(),
//             dims2: b.to_vec(),
//         };

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_volume1() -> ArrResult<()> {
//         let a = Shape::new([3, 256, 256])?;

//         let expected = 3 * 256 * 256;
//         let actual = a.volume();

//         Ok(assert_eq!(expected, actual))
//     }

//     #[test]
//     fn test_volume2() -> ArrResult<()> {
//         let a = Shape::new([1, 2, 3, 4, 5, 6, 7, 8, 9])?;

//         let expected = (1..10).product::<usize>();
//         let actual = a.volume();

//         Ok(assert_eq!(expected, actual))
//     }
// }

/*
mod derank_slice_tests {
    use crate::array::Error;

    #[test]
    fn test_derank_0() {
        arrs!(let arr = Array(vec![2, 2], vec![0, 1, 2, 3]));
        let actual = arr.derank(0).expect("deranking returned an error");
        arrs!(let expected = Array(vec![2], vec![0, 1]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_derank_1() {
        arrs!(let arr = Array(vec![2, 2], vec![0, 1, 2, 3]));
        let actual = arr.derank(1).expect("deranking returned an error");
        arrs!(let expected = Array(vec![2], vec![2, 3]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_derank_err_1d() {
        arrs!(let arr = Array(vec![2], vec![0, 1]));
        let result = arr.derank(1);
        assert_eq!(result.unwrap_err(), Error::Derank1D)
    }

    #[test]
    fn test_derank_err_invalid_index() {
        arrs!(let arr = Array(vec![2, 2], vec![0, 1, 2, 3]));
        let result = arr.derank(2);
        assert_eq!(
            result.unwrap_err(),
            Error::DerankIndexOutOfBounds { len: 2, index: 2 }
        )
    }

    #[test]
    fn test_slice_0_to_2() {
        arrs!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let actual = arr.slice(0, 2).expect("slicing returned an error");
        arrs!(let expected = Array(vec![2, 2], vec![0, 1, 2, 3]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_slice_1_to_3() {
        arrs!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let actual = arr.slice(1, 3).expect("slicing returned an error");
        arrs!(let expected = Array(vec![2, 2], vec![2, 3, 4, 5]));
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_slice_err_zero_width() {
        arrs!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let result = arr.slice(1, 1);
        assert_eq!(result.unwrap_err(), Error::SliceZeroWidth { index: 1 })
    }

    #[test]
    fn test_slice_err_stop_before_step() {
        arrs!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let result = arr.slice(2, 1);
        assert_eq!(
            result.unwrap_err(),
            Error::SliceStopBeforeStart { start: 2, stop: 1 }
        )
    }

    #[test]
    fn test_slice_err_stop_past_end() {
        arrs!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let result = arr.slice(2, 4);
        assert_eq!(
            result.unwrap_err(),
            Error::SliceStopPastEnd { stop: 4, dim: 3 }
        )
    }

    #[test]
    fn test_slice_deranked_1() {
        arrs!(let arr = Array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]));
        let sliced = arr.slice(1, 3).expect("slicing returned an error");
        let deranked = sliced.derank(0).expect("deranking returned an error");
        arrs!(let expected = Array(vec![2], vec![2, 3]));
        assert_eq!(expected, deranked)
    }
}
*/

/*
mod array_idx_tests {
    use crate::array::{ArrResult, Shape};

    #[test]
    fn read_correct_ndims() -> ArrResult<()> {
        arrs!(let _test_imgs = IDX("idx-files/t10k-images-idx3-ubyte"));
        Ok(())
    }

    #[test]
    fn read_correct_dims() -> ArrResult<()> {
        arrs!(let test_imgs = IDX("idx-files/t10k-images-idx3-ubyte"));
        let expected = Shape::new([10000, 28, 28])?;

        Ok(assert_eq!(&expected, test_imgs.shape()))
    }
}

*/
