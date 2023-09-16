@0xcbe8a53cbb941888;

struct Envelope {
  metadata @0 :Metadata;
  content @1 :Data;
}

struct Metadata {
  domain @0 :Text;
  entity @1 :Text;
  timestamp @2 :UInt64;
  sequence @3 :UInt64;
}
