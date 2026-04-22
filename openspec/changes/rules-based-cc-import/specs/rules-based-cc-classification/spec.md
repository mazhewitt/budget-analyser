## ADDED Requirements

### Requirement: Rules-based classifier for UBS credit card rows

The system SHALL provide a classifier that accepts a parsed credit-card CSV row (including `Booking text`, `Sector`, `Amount`, `Currency`, `Account/Cardholder`) and returns a `ClassificationResult` with `source = "rules"`, OR returns `None` to indicate the row should fall through to the existing LLM classifier. The classifier SHALL be pure (no I/O, no mutable state) and SHALL NOT perform any network calls.

#### Scenario: Sector maps to category deterministically
- **WHEN** a credit card row has `Sector = "Grocery stores"` and `Booking text = "Migros ZH Tiefe Zürich CHE"` with no matching merchant override
- **THEN** the rules classifier SHALL return a result with `category = "Groceries"`, `source = "rules"`, `confidence = 0.90`

#### Scenario: Empty sector with no override falls through to LLM
- **WHEN** a credit card row has an empty `Sector` field and no merchant override pattern matches `Booking text`
- **THEN** the rules classifier SHALL return `None` and the import pipeline SHALL invoke the existing LLM classifier for that row

#### Scenario: Unknown sector falls through to LLM
- **WHEN** a credit card row has `Sector = "<some future MCC not in the lookup table>"` and no merchant override matches
- **THEN** the rules classifier SHALL return `None` AND SHALL emit a warning log line naming the unmapped sector so the gap is visible

### Requirement: Merchant-string overrides take precedence over sector lookup

The classifier SHALL evaluate an ordered list of merchant-string regex patterns against `Booking text` BEFORE consulting the sector lookup table. The first matching pattern SHALL produce the classification result. The sector lookup SHALL only be consulted when no merchant override matches.

#### Scenario: Supermarket override beats Fast-Food sector
- **WHEN** a row has `Booking text = "Migros MR Brunaupark Zürich CHE"` with `Sector = "Fast-Food Restaurants"`
- **THEN** the override matching `/(?i)^(MIGROS|COOP|ALDI|LIDL|DENNER)/` SHALL fire first, producing `category = "Groceries"`, NOT `Dining`

#### Scenario: UBS staff cafeteria override beats Hotels sector
- **WHEN** a row has `Booking text = "UBS Rest. Flur Zürich Zürich CHE"` with `Sector = "Hotels"`
- **THEN** the override matching `/(?i)UBS Rest\./` SHALL fire, producing `category = "Dining"`, NOT `Travel`

### Requirement: TWINT peer-to-peer transfers to family initials map to Children

The classifier SHALL recognise TWINT P2P transfers whose recipient matches the family initial pattern `[KTL](.?H[A-Z]*.?)?` (first initial K, T, or L followed optionally by surname letters starting with H) and classify them as `category = "Children"`. Other TWINT `*Sent to` recipients SHALL be classified as `category = "Transfers"`.

#### Scenario: TWINT to L.H maps to Children
- **WHEN** a row has `Booking text = "TWINT *Sent to L.H 076***1234 CHE"`
- **THEN** the classifier SHALL return `merchant = "Family"`, `category = "Children"`

#### Scenario: TWINT to T.H maps to Children (surname initial form)
- **WHEN** a row has `Booking text = "TWINT *Sent to T.H 078***5678 CHE"`
- **THEN** the classifier SHALL return `category = "Children"`

#### Scenario: TWINT to T.P does NOT match family pattern
- **WHEN** a row has `Booking text = "TWINT *Sent to T.P 079***9999 CHE"` (different surname)
- **THEN** the classifier SHALL NOT classify as Children; it SHALL fall through to the generic TWINT-P2P override which produces `category = "Transfers"`

#### Scenario: TWINT UBS top-up maps to Transfers
- **WHEN** a row has `Booking text = "TWINT *UBS TWINT Zürich CHE"` (no "Sent to")
- **THEN** the classifier SHALL return `category = "Transfers"` as a card top-up

### Requirement: Sector-to-category lookup table

The system SHALL maintain a hand-curated mapping from UBS Sector strings to budget categories. The mapping SHALL be defined in source code (not derived from database distributions). The mapping SHALL cover at minimum the sectors observed in historical CC imports; new sectors encountered in future imports SHALL log a warning and fall through to LLM.

The mapping SHALL use the existing 18-category schema. Canonical entries include:

| Sector | Category |
|---|---|
| Restaurants, Fast Food Restaurant, Fast-Food Restaurants, Caterers, Bakeries | Dining |
| Grocery stores, Candy and nut stores, Package stores - beer, Freezer and locker meat provisioners | Groceries |
| Hotels, Travel agencies, Tourist Attractions and Exhibits, Camp grounds, Aparments | Travel |
| Airlines, British Airways, Swiss International Air Lines, KLM, Lufthansa, LOT (Poland), Airports | Travel |
| Commuter transportation, Passenger railways, Bus lines, Taxicabs, Gasoline service stations, Parking & Garages, Automobile services, Toll and bridge fees, Car Rental Company, Fines | Transport |
| Pharmacies, Doctors and Physicians, Hospitals, Dentists and Orthodontists, Optician | Healthcare |
| Clothing store, Clothing - sports, Shoe stores, Department stores, Furniture, Electronics Stores, Home supply warehouse stores, Hardware stores, Cosmetic stores, Book stores, Office supply stores, Secondhand stores, Household appliance stores, Garden and hardware center, Games and hobby stores, Leather goods, Jewelry stores, Clock or jewelry or watch stores, Non-durable Goods (B2B), Retail business | Shopping |
| Digital goods, Computer software stores, Computer network/Information services, Continuity / Subscription Merchant, Membership Organizations, Misc. publishing and printing services, Books & newspapers (B2B), Telegraph services, Data processing services, Films / Video production / distribution, Bands Orchestras & Music Entertainment | Subscriptions |
| Government Services, Postal Services, Money orders - wire transfer | Fees |
| Schools and Educational Services | Children |
| Barber or beauty shops, Cleaning - laundry and garment services | Shopping |
| Theather Production / Ticket Agencies, Cinema, Recreation Services, Commercial Sports | Other |

#### Scenario: Every observed historical sector has a mapping
- **WHEN** the rules classifier is tested against a sample of every non-empty Sector value present in the DB (as of the most recent CC import)
- **THEN** every such Sector SHALL produce a non-`None` result for at least one representative row

### Requirement: Deterministic merchant-name normalisation

The classifier SHALL produce a human-readable merchant name by normalising `Booking text` via the following steps, applied in order:

1. Strip trailing ISO-3 country codes (CHE, GBR, USA, IRL, DEU, NLD, ITA, FRA, SWE, HUN, POL, etc.) preceded by whitespace
2. Strip trailing location tails (repeated whitespace followed by a city name or postcode pattern)
3. Collapse runs of 2+ whitespace characters to a single space
4. Strip trailing card-number masks (`X{3,}`, `\*{3,}`)
5. Trim leading/trailing whitespace and punctuation

When a merchant override fires, the override MAY supply its own `merchant` value (e.g., "Family" for kid TWINTs) instead of running normalisation.

#### Scenario: Country code and padding stripped
- **WHEN** `Booking text = "Phills BBQ              Cham         CHE"`
- **THEN** the normalised merchant SHALL be `"Phills BBQ Cham"` (country code removed, whitespace collapsed)

#### Scenario: Card mask stripped
- **WHEN** `Booking text = "MERCHANT NAME XXXXXXXXXX 1234 CHE"`
- **THEN** the normalised merchant SHALL be `"MERCHANT NAME"` (card mask and country code removed)

### Requirement: Rules path bypasses the merchant cache

When the rules classifier produces a result for a credit card row, the import pipeline SHALL NOT perform a cache lookup BEFORE calling the rules classifier, and SHALL NOT write the rules result into the merchant cache. The cache remains in use for the LLM fallback path and for non-CC import formats.

#### Scenario: No cache lookup on rules path
- **WHEN** a CC row is classified via rules (merchant override or sector lookup)
- **THEN** the import pipeline SHALL NOT call `db.cache_lookup(...)` for that row

#### Scenario: Cache write skipped on rules path
- **WHEN** a CC row is classified via rules
- **THEN** the import pipeline SHALL NOT call `db.cache_insert(...)` for that row

#### Scenario: LLM fallback still uses cache
- **WHEN** a CC row falls through to LLM (empty sector AND no override match)
- **THEN** the import pipeline SHALL perform the existing cache lookup and, on miss, write the LLM result to the cache
