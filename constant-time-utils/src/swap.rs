use subtle::{Choice, ConditionallySelectable};

use crate::cmp::ct_lt;

pub unsafe fn compare_and_swap(
    base: *mut [u8; 32],
    i: usize,
    j: usize,
    dir_asc: bool,
) {
    let left = unsafe { *base.add(i) };
    let right = unsafe { *base.add(j) };

    let should_swap = if dir_asc {
        ct_lt(&right, &left)
    } else {
        ct_lt(&left, &right)
    };
    let choice = Choice::from(should_swap as u8);

    let mut new_left = [0u8; 32];
    let mut new_right = [0u8; 32];
    for idx in 0..32 {
        new_left[idx] = u8::conditional_select(&left[idx], &right[idx], choice);
        new_right[idx] = u8::conditional_select(&right[idx], &left[idx], choice);
    }

    unsafe {
        *base.add(i) = new_left;
        *base.add(j) = new_right;
    }
}

#[cfg(test)]
mod tests {
    use super::compare_and_swap;

    #[test]
    fn swaps_for_ascending_order() {
        let mut data = [[0u8; 32]; 2];
        data[0][0] = 5;
        data[1][0] = 3;

        unsafe {
            compare_and_swap(data.as_mut_ptr(), 0, 1, true);
        }

        assert_eq!(data[0][0], 3);
        assert_eq!(data[1][0], 5);
    }

    #[test]
    fn swaps_for_descending_order() {
        let mut data = [[0u8; 32]; 2];
        data[0][0] = 1;
        data[1][0] = 9;

        unsafe {
            compare_and_swap(data.as_mut_ptr(), 0, 1, false);
        }

        assert_eq!(data[0][0], 9);
        assert_eq!(data[1][0], 1);
    }

    #[test]
    fn keeps_order_when_already_correct() {
        let mut data = [[0u8; 32]; 2];
        data[0][0] = 1;
        data[1][0] = 4;

        unsafe {
            compare_and_swap(data.as_mut_ptr(), 0, 1, true);
        }

        assert_eq!(data[0][0], 1);
        assert_eq!(data[1][0], 4);
    }
}
