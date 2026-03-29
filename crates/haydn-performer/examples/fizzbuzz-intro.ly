% FizzBuzz opening measures — the push-compare-branch pattern
%
% Demonstrates what control flow "sounds like" on the piano tuning.
% The pattern: push value, push divisor, mod, compare, conditional branch
% creates a rhythmic alternation between low and high registers.
%
% Piano tuning reference:
%   Value zone: MIDI 36-59 (C2-B3) → Push(midi_note - 60)
%   mod = B4 (MIDI 71), eq = E5 (76), gt = F5 (77)
%   print_char = A5 (81), print_num = B5 (83)
%   loop_start = C5 (72), loop_end = D5 (74)

% Push 1, start a loop
c,4 c''4

% Push counter value, push 3, mod, check
e,4 d,4
b'4 e''4

% Push counter value, push 5, mod, check
e,4 a,4
b'4 e''4

% Print the number
b''4

% Loop back
d''4

% Rest between repetitions
r2

% Another iteration — hear the repeating pattern
e,4 d,4 b'4 e''4
e,4 a,4 b'4 e''4
b''4 d''4
