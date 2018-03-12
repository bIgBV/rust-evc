/// Test for primality of an unsigned number
/// 
///```
/// assert!(is_prime(29));
///```
fn is_prime(n: u64) -> bool {
    if n <= 1 {
        return false;
    } else if n <= 3 {
        return true;
    } else if n % 2 == 0 || n % 3 == 0 {
        return false;
    } else {
        let mut i = 5;
        while (i * i) <= n {
            if (n % i) == 0 || (n % (i + 2)) == 0 {
                return false;
            }
            i += 6;
        }
    }
    true
}

#[test]
fn test_is_prime() {
    assert!(!is_prime(100));
    assert!(is_prime(17));
    assert!(!is_prime(12));
}

/// Generate the nth prime number
/// 
/// ```
/// assert_eq!(nth_prime(15), 47)
/// ```
pub fn nth_prime(n: u64) -> u64 {
    let mut i = 1;
    let mut counter = 1;

    while counter <= n {
        i += 1;
        if is_prime(i) {
            counter += 1;
        }
    }

    return i
}

#[test]
fn test_nth_prime() {
    assert_eq!(nth_prime(10), 29);
    assert_eq!(nth_prime(100), 541);
}