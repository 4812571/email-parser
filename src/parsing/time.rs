use crate::prelude::*;

pub type Zone = (bool, u8, u8);
pub type Time = ((u8, u8, u8), Zone);
pub type Date = (usize, Month, usize);
pub type DateTime = (Option<Day>, Date, Time);

#[derive(Debug, PartialEq)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

pub fn day_name(input: &[u8]) -> Res<Day> {
    if let (Some(input), Some(letters)) = (input.get(3..), input.get(..3)) {
        let letters = letters.to_ascii_lowercase();
        match letters.as_slice() {
            b"mon" => Ok((input, Day::Monday)),
            b"tue" => Ok((input, Day::Tuesday)),
            b"wed" => Ok((input, Day::Wednesday)),
            b"thu" => Ok((input, Day::Thursday)),
            b"fri" => Ok((input, Day::Friday)),
            b"sat" => Ok((input, Day::Saturday)),
            b"sun" => Ok((input, Day::Sunday)),
            _ => Err(Error::Known("Not a valid day_name")),
        }
    } else {
        Err(Error::Known(
            "Expected day_name, but characters are missing (at least 3).",
        ))
    }
}

pub fn month(input: &[u8]) -> Res<Month> {
    if let (Some(input), Some(letters)) = (input.get(3..), input.get(..3)) {
        let letters = letters.to_ascii_lowercase();
        match letters.as_slice() {
            b"jan" => Ok((input, Month::January)),
            b"feb" => Ok((input, Month::February)),
            b"mar" => Ok((input, Month::March)),
            b"apr" => Ok((input, Month::April)),
            b"may" => Ok((input, Month::May)),
            b"jun" => Ok((input, Month::June)),
            b"jul" => Ok((input, Month::July)),
            b"aug" => Ok((input, Month::August)),
            b"sep" => Ok((input, Month::September)),
            b"oct" => Ok((input, Month::October)),
            b"nov" => Ok((input, Month::November)),
            b"dec" => Ok((input, Month::December)),
            _ => Err(Error::Known("Not a valid month")),
        }
    } else {
        Err(Error::Known(
            "Expected month, but characters are missing (at least 3).",
        ))
    }
}

pub fn day_of_week(input: &[u8]) -> Res<Day> {
    let (input, _fws) = optional(input, fws);
    let (input, day) = day_name(input)?;
    let (input, ()) = tag(input, b",")?;
    Ok((input, day))
}

pub fn year(input: &[u8]) -> Res<usize> {
    let (input, _) = fws(input)?;

    let (input, year) =
        take_while1(input, is_digit).map_err(|_e| Error::Known("no digit in year"))?;
    if year.len() < 4 {
        return Err(Error::Known("year is expected to have 4 digits or more"));
    }
    let year: usize = as_str(&year)
        .parse()
        .map_err(|_e| Error::Known("Failed to parse year"))?;

    if year < 1990 {
        return Err(Error::Known("year must be after 1990"));
    }

    let (input, _) = fws(input)?;

    Ok((input, year))
}

pub fn day(input: &[u8]) -> Res<usize> {
    let (input, _fws) = optional(input, fws);
    let (mut input, mut day) = digit(input)?;
    if let Ok((new_input, digit)) = digit(input) {
        day *= 10;
        day += digit;
        input = new_input;
    }
    if day > 31 {
        return Err(Error::Known("day must be less than 31"));
    }
    let (input, _) = fws(input)?;
    Ok((input, day as usize))
}

pub fn time_of_day(input: &[u8]) -> Res<(u8, u8, u8)> {
    let (input, hour) = two_digits(input)?;
    if hour > 23 {
        return Err(Error::Known("There is only 24 hours in a day"));
    }
    let (input, ()) = tag(input, b":")?;

    let (input, minutes) = two_digits(input)?;
    if minutes > 59 {
        return Err(Error::Known("There is only 60 minutes per hour"));
    }

    if input.starts_with(b":") {
        let new_input = &input[1..];
        if let Ok((new_input, seconds)) = two_digits(new_input) {
            if seconds > 60 {
                // leap second allowed
                return Err(Error::Known("There is only 60 seconds in a minute"));
            }
            return Ok((new_input, (hour, minutes, seconds)));
        }
    }

    Ok((input, (hour, minutes, 0)))
}

pub fn zone(input: &[u8]) -> Res<Zone> {
    let (mut input, _fws) = fws(input)?;

    let sign = match input.get(0) {
        Some(b'+') => true,
        Some(b'-') => false,
        None => return Err(Error::Known("Expected more characters in zone")),
        _ => return Err(Error::Known("Invalid sign character in zone")),
    };
    input = &input[1..];

    let (input, hours) = two_digits(input)?;
    let (input, minutes) = two_digits(input)?;

    if minutes > 59 {
        return Err(Error::Known("zone minutes out of range"));
    }

    Ok((input, (sign, hours, minutes)))
}

pub fn time(input: &[u8]) -> Res<Time> {
    let (input, time_of_day) = time_of_day(input)?;
    let (input, zone) = zone(input)?;
    Ok((input, (time_of_day, zone)))
}

pub fn date(input: &[u8]) -> Res<Date> {
    let (input, day) = day(input)?;
    let (input, month) = month(input)?;
    let (input, year) = year(input)?;
    Ok((input, (day, month, year)))
}

pub fn date_time(input: &[u8]) -> Res<DateTime> {
    let (input, day) = optional(input, day_of_week);
    let (input, date) = date(input)?;
    let (input, time) = time(input)?;
    let (input, _cfws) = optional(input, cfws);
    Ok((input, (day, date, time)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_day() {
        assert_eq!(day_name(b"Mon ").unwrap().1, Day::Monday);
        assert_eq!(day_name(b"moN ").unwrap().1, Day::Monday);
        assert_eq!(day_name(b"thu").unwrap().1, Day::Thursday);

        assert_eq!(day_of_week(b"   thu, ").unwrap().1, Day::Thursday);
        assert_eq!(day_of_week(b"wed, ").unwrap().1, Day::Wednesday);
        assert_eq!(day_of_week(b" Sun,").unwrap().1, Day::Sunday);

        assert_eq!(day(b"31 ").unwrap().1, 31);
        assert_eq!(day(b"9 ").unwrap().1, 9);
        assert_eq!(day(b"05 ").unwrap().1, 5);
        assert_eq!(day(b"23 ").unwrap().1, 23);
    }

    #[test]
    fn test_month_and_year() {
        assert_eq!(month(b"Apr ").unwrap().1, Month::April);
        assert_eq!(month(b"may ").unwrap().1, Month::May);
        assert_eq!(month(b"deC ").unwrap().1, Month::December);

        assert_eq!(year(b" 2020 ").unwrap().1, 2020);
        assert_eq!(year(b"\r\n 1995 ").unwrap().1, 1995);
        assert_eq!(year(b" 250032 ").unwrap().1, 250032);
    }

    #[test]
    fn test_date() {
        assert_eq!(date(b"1 nov 2020 ").unwrap().1, (1, Month::November, 2020));
        assert_eq!(
            date(b"25 dec 2038 ").unwrap().1,
            (25, Month::December, 2038)
        );

        assert_eq!(
            date_time(b"Mon, 12 Apr 2023 10:25:03 +0000").unwrap().1,
            (
                Some(Day::Monday),
                (12, Month::April, 2023),
                ((10, 25, 3), (true, 0, 0))
            )
        );
        assert_eq!(
            date_time(b"5 May 2003 18:59:03 +0000").unwrap().1,
            (None, (5, Month::May, 2003), ((18, 59, 3), (true, 0, 0)))
        );
    }

    #[test]
    fn test_time() {
        assert_eq!(time_of_day(b"10:40:29").unwrap().1, (10, 40, 29));
        assert_eq!(time_of_day(b"10:40 ").unwrap().1, (10, 40, 0));
        assert_eq!(time_of_day(b"05:23 ").unwrap().1, (5, 23, 0));

        assert_eq!(zone(b" +1000 ").unwrap().1, (true, 10, 0));
        assert_eq!(zone(b" -0523 ").unwrap().1, (false, 5, 23));

        assert_eq!(time(b"06:44 +0100").unwrap().1, ((6, 44, 0), (true, 1, 0)));
        assert_eq!(time(b"23:57 +0000").unwrap().1, ((23, 57, 0), (true, 0, 0)));
        assert_eq!(
            time(b"08:23:02 -0500").unwrap().1,
            ((8, 23, 2), (false, 5, 0))
        );
    }
}
