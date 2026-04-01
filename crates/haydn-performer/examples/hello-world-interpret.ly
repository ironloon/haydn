% "Hello, World!" — a real Haydn program performed as music
%
% Each character's ASCII code is built from value-zone notes (C2-B3),
% operated on with high-register notes (C4+), then printed with A5.
% Run with: haydn-performer hello-world-interpret.ly --interpret --tuning piano.toml
%
% Piano tuning zones:
%   MIDI 36-59 (C2-B3): Push(midi - 60), values -24 to -1
%   MIDI 60 (C4): add    MIDI 62 (D4): sub    MIDI 67 (G4): mul
%   MIDI 81 (A5): print_char

% H (72): (-8) × (-9) = 72
e4 dis4 g'4 a''4
r4

% e (101): (-10) × (-10) - (-1) = 101
d4 d4 g'4 b4 d'4 a''4
r4

% l (108): (-12) × (-9) = 108
c4 dis4 g'4 a''4

% l (108): (-12) × (-9) = 108
c4 dis4 g'4 a''4
r4

% o (111): (-12) × (-10) + (-9) = 111
c4 d4 g'4 dis4 c'4 a''4
r4

% , (44): (-4) × (-11) = 44
gis4 cis4 g'4 a''4

% (space 32): (-4) × (-8) = 32
gis4 e4 g'4 a''4
r4

% W (87): (-9) × (-10) + (-3) = 87
dis4 d4 g'4 a4 c'4 a''4
r4

% o (111): (-12) × (-10) + (-9) = 111
c4 d4 g'4 dis4 c'4 a''4
r4

% r (114): (-6) × (-19) = 114
fis4 f,4 g'4 a''4

% l (108): (-12) × (-9) = 108
c4 dis4 g'4 a''4

% d (100): (-10) × (-10) = 100
d4 d4 g'4 a''4
r4

% ! (33): (-3) × (-11) = 33
a4 cis4 g'4 a''4

% newline (10): (-2) × (-5) = 10
ais4 g4 g'4 a''4
