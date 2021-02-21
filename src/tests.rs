mod array_tests {
    use crate::array::{Array, Data, Shape, TypeAware};

    fn new_array<T>(shape: Vec<isize>, data: Vec<T>) -> Array<T>
    where
        T: TypeAware,
    {
        let shape = Shape::new(shape);
        let data = Data::new(data);

        Array::new(shape, data).expect("Data doesn't contain correct number of items for Shape")
    }

    #[test]
    fn test_eq1() {
        let arr1 = new_array(vec![2, 2], vec![0, 1, 2, 3]);
        let arr2 = new_array(vec![2, 2], vec![0, 1, 2, 3]);
        assert_eq!(arr1, arr2);
    }

    #[test]
    fn test_cast1() {
        let arr1 = new_array(vec![1], vec![10]);
        let arr2 = new_array(vec![4], vec![0, 1, 2, 3]);
        let expected = new_array(vec![4], vec![0, 10, 20, 30]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast2() {
        let arr1 = new_array(vec![1], vec![10]);
        let arr2 = new_array(vec![2, 2], vec![0, 1, 2, 3]);
        let expected = new_array(vec![2, 2], vec![0, 10, 20, 30]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast3() {
        let arr1 = new_array(vec![2], vec![0, 1]);
        let arr2 = new_array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let expected = new_array(vec![2, 3], vec![0, 1, 0, 3, 0, 5]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast4() {
        let arr1 = new_array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let arr2 = new_array(vec![1, 2], vec![0, 1]);
        let expected = new_array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast5() {
        let arr1 = new_array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let arr2 = new_array(vec![1, 2], vec![0, 1]);
        let expected = new_array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }
}

mod serde_arrs_tests {
    use crate::array::Shape;
    use crate::serde_arrs;

    #[test]
    fn read_correct_ndims() {
        let test_imgs = serde_arrs::from_idx::<u8>("idx-files/t10k-images-idx3-ubyte").unwrap();
        assert_eq!(test_imgs.ndims(), 3)
    }

    #[test]
    fn read_correct_dims() {
        let test_imgs = serde_arrs::from_idx::<u8>("idx-files/t10k-images-idx3-ubyte").unwrap();

        let expected_shape = Shape::new(vec![10000, 28, 28]);
        assert_eq!(&expected_shape, test_imgs.shape())
    }
}

mod shape_tests {
    use crate::array::{Error, Shape};

    #[test]
    fn test_eq1() {
        let a = Shape::new(vec![3, 4, 5]);
        let b = Shape::new(vec![3, 4, 5]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_eq2() {
        let a = Shape::new(vec![1000, 1000, 1000]);
        let b = Shape::new(vec![1000, 1000, 1000]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne1() {
        let a = Shape::new(vec![1000, 1000, 1000]);
        let b = Shape::new(vec![1001, 1000, 1000]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_ne2() {
        let a = Shape::new(vec![1, 2, 3]);
        let b = Shape::new(vec![1, 2]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_cast_ok1() {
        let a = Shape::new(vec![3, 256, 256]);
        let b = Shape::new(vec![3]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![3, 256, 256]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok2() {
        let a = Shape::new(vec![1, 6, 1, 8]);
        let b = Shape::new(vec![5, 1, 7]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 6, 7, 8]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok3() {
        let a = Shape::new(vec![4, 5]);
        let b = Shape::new(vec![1]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![4, 5]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok4() {
        let a = Shape::new(vec![4, 5]);
        let b = Shape::new(vec![4]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![4, 5]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok5() {
        let a = Shape::new(vec![5, 3, 15]);
        let b = Shape::new(vec![5, 1, 15]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok6() {
        let a = Shape::new(vec![5, 3, 15]);
        let b = Shape::new(vec![5, 3]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok7() {
        let a = Shape::new(vec![5, 3, 15]);
        let b = Shape::new(vec![1, 3]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_err1() {
        let a = Shape::new(vec![3]);
        let b = Shape::new(vec![4]);
        let res = a.cast(&b);
        let expected = Error::Cast(a.clone(), b.clone());
        assert_eq!(expected, res.unwrap_err());
    }

    #[test]
    fn test_cast_err2() {
        let a = Shape::new(vec![1, 2]);
        let b = Shape::new(vec![3, 4, 8]);
        let res = a.cast(&b);
        let expected = Error::Cast(a.clone(), b.clone());
        assert_eq!(expected, res.unwrap_err());
    }

    #[test]
    fn test_volume1() {
        let a = Shape::new(vec![3, 256, 256]);
        let expected = 3 * 256 * 256;
        assert_eq!(expected, a.volume());
    }

    #[test]
    fn test_volume2() {
        let a = Shape::new((1..11).collect::<Vec<isize>>());
        let expected = (1..11).product::<usize>();
        assert_eq!(expected, a.volume());
    }

    #[test]
    fn test_display1() {
        let a = Shape::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(a.to_string(), String::from("[5, 4, 3, 2, 1]"));
    }
}
