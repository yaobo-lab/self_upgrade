use semver::Version;
use toolkit_rs::AppResult;

pub fn is_greater(current: &str, other: &str) -> AppResult<bool> {
    Ok(Version::parse(other)? > Version::parse(current)?)
}

pub fn is_compatible(current: &str, other: &str) -> AppResult<bool> {
    let current = Version::parse(current)?;
    let other = Version::parse(other)?;
    Ok(if !current.pre.is_empty() {
        current.major == other.major
            && ((other.minor >= current.minor)
                || (current.minor == other.minor && other.patch >= current.patch))
    } else if other.major == 0 && current.major == 0 {
        current.minor == other.minor && other.patch > current.patch && other.pre.is_empty()
    } else if other.major > 0 {
        current.major == other.major
            && ((other.minor > current.minor)
                || (current.minor == other.minor && other.patch > current.patch))
            && other.pre.is_empty()
    } else {
        false
    })
}

pub fn is_major(current: &str, other: &str) -> AppResult<bool> {
    let current = Version::parse(current)?;
    let other = Version::parse(other)?;
    Ok(other.major > current.major)
}

pub fn is_minor(current: &str, other: &str) -> AppResult<bool> {
    let current = Version::parse(current)?;
    let other = Version::parse(other)?;
    Ok(current.major == other.major && other.minor > current.minor)
}

pub fn is_patch(current: &str, other: &str) -> AppResult<bool> {
    let current = Version::parse(current)?;
    let other = Version::parse(other)?;
    Ok(current.major == other.major && current.minor == other.minor && other.patch > current.patch)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_greater() {
        assert!(is_greater("1.2.0", "1.2.3").unwrap());
        assert!(is_greater("0.2.0", "1.2.3").unwrap());
        assert!(is_greater("0.2.0", "0.2.3").unwrap());
    }

    #[test]
    fn test_is_compatible() {
        assert!(!is_compatible("1.2.0", "2.3.1").unwrap());
        assert!(!is_compatible("0.2.0", "2.3.1").unwrap());
        assert!(!is_compatible("1.2.3", "3.3.0").unwrap());
        assert!(!is_compatible("1.2.3", "0.2.0").unwrap());
        assert!(!is_compatible("0.2.0", "0.3.0").unwrap());
        assert!(!is_compatible("0.3.0", "0.2.0").unwrap());
        assert!(!is_compatible("1.2.3", "1.1.0").unwrap());
        assert!(!is_compatible("2.0.0", "2.0.0-alpha.1").unwrap());
        assert!(!is_compatible("1.2.3", "2.0.0-alpha.1").unwrap());
        assert!(!is_compatible("2.0.0-alpha.1", "3.0.0").unwrap());

        assert!(is_compatible("1.2.0", "1.2.3").unwrap());
        assert!(is_compatible("0.2.0", "0.2.3").unwrap());
        assert!(is_compatible("1.2.0", "1.3.3").unwrap());
        assert!(is_compatible("2.0.0-alpha.0", "2.0.0-alpha.1").unwrap());
        assert!(is_compatible("2.0.0-alpha.0", "2.0.0").unwrap());
        assert!(is_compatible("2.0.0-alpha.0", "2.0.1").unwrap());
        assert!(is_compatible("2.0.0-alpha.0", "2.1.0").unwrap());
    }

    #[test]
    fn test_is_major() {
        assert!(is_major("1.2.0", "2.3.1").unwrap());
        assert!(is_major("0.2.0", "2.3.1").unwrap());
        assert!(is_major("1.2.3", "3.3.0").unwrap());
        assert!(!is_major("1.2.3", "1.2.0").unwrap());
        assert!(!is_major("1.2.3", "0.2.0").unwrap());
    }

    #[test]
    fn test_is_minor() {
        assert!(!is_minor("1.2.0", "2.3.1").unwrap());
        assert!(!is_minor("0.2.0", "2.3.1").unwrap());
        assert!(!is_minor("1.2.3", "3.3.0").unwrap());
        assert!(is_minor("1.2.3", "1.3.0").unwrap());
        assert!(is_minor("0.2.3", "0.4.0").unwrap());
    }

    #[test]
    fn test_is_patch() {
        assert!(!is_patch("1.2.0", "2.3.1").unwrap());
        assert!(!is_patch("0.2.0", "2.3.1").unwrap());
        assert!(!is_patch("1.2.3", "3.3.0").unwrap());
        assert!(!is_patch("1.2.3", "1.2.3").unwrap());
        assert!(is_patch("1.2.0", "1.2.3").unwrap());
        assert!(is_patch("0.2.3", "0.2.4").unwrap());
    }
}
