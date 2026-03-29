% "Hello World" — notes that exercise the piano tuning's value + operation zones
%
% The piano tuning maps:
%   MIDI 36-59 (C2-B3): Push values (midi_note - 60)
%   MIDI 60+ (C4+):     Operations (add, sub, dup, etc.)
%
% This sequence demonstrates what "programming sounds like" on the default
% piano tuning — low notes push values, high notes operate on them.
% The contrast between the bass value zone and treble operation zone
% creates a distinctive musical character.
%
% Pattern: push values (low), then operate (high), repeat

% Push some values in the value zone (C2-B3)
c,4 e,4 g,4 c4
% Operations: add (C4), dup (E4), print_char (A5)
c'4 e'4 a''4

% More values
d,4 f,4 a,4 d4
% Operations: mul (G4), print_char (A5)
g'4 a''4

% Ascending value sequence
c,8 d,8 e,8 f,8 g,8 a,8 b,8 c8
% Operation burst: dup, add, dup, add, print_num (B5)
e'8 c'8 e'8 c'8 b''4

% Rest between phrases
r2

% Final: descending values with operations interleaved
b4 a4 g4 f4
c'4 d'4
e4 d4 c4
a''2
