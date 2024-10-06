use grep::{
    matcher::{Match, Matcher},
    searcher::{Searcher, SinkError},
};
use std::io;

// The maximum number of bytes to execute a search to account for look-ahead.
//
// This is an unfortunate kludge since PCRE2 doesn't provide a way to search
// a substring of some input while accounting for look-ahead. In theory, we
// could refactor the various 'grep' interfaces to account for it, but it would
// be a large change. So for now, we just let PCRE2 go looking a bit for a
// match without searching the entire rest of the contents.
//
// Note that this kludge is only active in multi-line mode.
const MAX_LOOK_AHEAD: usize = 128;

pub fn find_iter_at_in_context<M, F>(
    searcher: &Searcher,
    matcher: M,
    mut bytes: &[u8],
    range: std::ops::Range<usize>,
    mut matched: F,
) -> io::Result<()>
where
    M: Matcher,
    F: FnMut(Match) -> bool,
{
    // This strange dance is to account for the possibility of look-ahead in
    // the regex. The problem here is that mat.bytes() doesn't include the
    // lines beyond the match boundaries in mulit-line mode, which means that
    // when we try to rediscover the full set of matches here, the regex may no
    // longer match if it required some look-ahead beyond the matching lines.
    //
    // PCRE2 (and the grep-matcher interfaces) has no way of specifying an end
    // bound of the search. So we kludge it and let the regex engine search the
    // rest of the buffer... But to avoid things getting too crazy, we cap the
    // buffer.
    //
    // If it weren't for multi-line mode, then none of this would be needed.
    // Alternatively, if we refactored the grep interfaces to pass along the
    // full set of matches (if available) from the searcher, then that might
    // also help here. But that winds up paying an upfront unavoidable cost for
    // the case where matches don't need to be counted. So then you'd have to
    // introduce a way to pass along matches conditionally, only when needed.
    // Yikes.
    //
    // Maybe the bigger picture thing here is that the searcher should be
    // responsible for finding matches when necessary, and the printer
    // shouldn't be involved in this business in the first place. Sigh. Live
    // and learn. Abstraction boundaries are hard.
    let is_multi_line = searcher.multi_line_with_matcher(&matcher);
    if is_multi_line {
        if bytes[range.end..].len() >= MAX_LOOK_AHEAD {
            bytes = &bytes[..range.end + MAX_LOOK_AHEAD];
        }
    } else {
        // When searching a single line, we should remove the line terminator.
        // Otherwise, it's possible for the regex (via look-around) to observe
        // the line terminator and not match because of it.
        let mut m = Match::new(0, range.end);
        trim_line_terminator(searcher, bytes, &mut m);
        bytes = &bytes[..m.end()];
    }
    matcher
        .find_iter_at(bytes, range.start, |m| {
            if m.start() >= range.end {
                return false;
            }
            matched(m)
        })
        .map_err(io::Error::error_message)
}

/// Given a buf and some bounds, if there is a line terminator at the end of
/// the given bounds in buf, then the bounds are trimmed to remove the line
/// terminator.
pub fn trim_line_terminator(searcher: &Searcher, buf: &[u8], line: &mut Match) {
    let lineterm = searcher.line_terminator();
    if lineterm.is_suffix(&buf[*line]) {
        let mut end = line.end() - 1;
        if lineterm.is_crlf() && end > 0 && buf.get(end - 1) == Some(&b'\r') {
            end -= 1;
        }
        *line = line.with_end(end);
    }
}
