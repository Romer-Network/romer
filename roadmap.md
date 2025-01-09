# Romer Chain 

## Story 1: Network Storage Initialization

### Background

Before we can handle account registration, our network needs fundamental data structures for tracking organizations and tokens. We implement these using Commonware's storage capabilities, ensuring consistent and efficient data management across the network.

Currently we need to store ROMER token and ROMER organisation in commonware_storage::metadata

## Story 2: Market Maker Registration Client

As a market maker  
I want to register my organization through a terminal interface  
So that I can prepare for trading operations  

### Background

Market makers need a straightforward way to register their organizations and establish their network identity. We'll create this using ratatui for a clean terminal user interface that guides them through the registration process.

### Technical Implementation

The story involves creating a ratatui-based client interface in our client crate:

Registration Interface Requirements:
- Organization name input
- Organization type selection
- SenderCompID input (with uniqueness verification)
- BLS key pair generation
- Registration confirmation display

Layout Components:
```rust
pub struct RegistrationForm {
    // Form fields
    pub name_input: Input,
    pub sender_comp_id_input: Input,
    pub org_type_select: Select,
    // Form state
    pub current_field: usize,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
}
```

### Success Criteria

- Clean, intuitive terminal interface implemented
- Real-time SenderCompID uniqueness verification
- Secure BLS key generation and storage
- Clear error handling and user feedback
- Successful organization registration in network storage

## Story 3: Stablecoin Issuer Registration Client

As a stablecoin issuer  
I want to register my organization and define my token  
So that I can prepare to issue stablecoins on the network  

### Background

Stablecoin issuers require similar registration capabilities to market makers, but with additional functionality for defining their tokens. The registration process must ensure both organization and token details are properly recorded.

### Technical Implementation

Extends the registration interface to include token definition:

Token Definition Interface:
```rust
pub struct TokenDefinitionForm {
    // Token details
    pub name_input: Input,
    pub symbol_input: Input,
    pub decimals_input: Input,
    // Form state
    pub current_field: usize,
    pub validation_errors: Vec<String>,
}
```

Registration Flow:
1. Organization registration (similar to market maker)
2. Token definition
3. Registration confirmation

### Success Criteria

- Organization registration functions identical to market maker
- Token definition interface implemented
- Token parameters validated
- Token successfully registered in network storage
- Clear feedback throughout process

## Implementation Notes

### Common Components Needed

1. Key Management:
   - BLS key generation using Commonware cryptography
   - Secure key storage
   - Key backup functionality

2. Network Storage:
   - Organization registry
   - Token registry
   - Uniqueness enforcement
   - Query optimization

3. User Interface:
   - Form management
   - Input validation
   - Error handling
   - Status updates

### Testing Requirements

1. Storage Tests:
   - Uniqueness constraints
   - CRUD operations
   - Query performance
   - Error conditions

2. Interface Tests:
   - Input validation
   - Error handling
   - Success flows
   - Edge cases

3. Integration Tests:
   - End-to-end registration
   - Key generation
   - Storage persistence
   - Duplicate detection

### Security Considerations

1. Key Generation:
   - Secure randomness
   - Key protection
   - Backup procedures

2. Data Validation:
   - Input sanitization
   - Uniqueness verification
   - Format validation

3. Storage Security:
   - Access controls
   - Data integrity
   - Audit logging

These stories establish the foundation for Romer Chain's account management and token issuance capabilities. They focus on creating a robust, secure, and user-friendly registration process for both market makers and stablecoin issuers.
