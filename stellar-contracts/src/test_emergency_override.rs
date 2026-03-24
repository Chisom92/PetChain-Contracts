use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_owner_can_read_emergency_info() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Private,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Emergency Name"),
        phone: String::from_str(&env, "555-1234"),
        email: String::from_str(&env, "emergency@test.com"),
        relationship: String::from_str(&env, "Friend"),
        is_primary: true,
    });

    let mut allergies = Vec::new(&env);
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Peanuts"),
        severity: String::from_str(&env, "High"),
        is_critical: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &allergies,
        &String::from_str(&env, "Critical medical condition!"),
    );

    // Owner can always read their own pet's emergency info
    let info = client.get_emergency_info(&pet_id, &owner);

    assert_eq!(info.pet_id, pet_id);
    assert_eq!(info.species, String::from_str(&env, "Dog"));
    assert_eq!(info.emergency_contacts.len(), 1);
    assert_eq!(
        info.emergency_contacts.get(0).unwrap().phone,
        String::from_str(&env, "555-1234")
    );
}

#[test]
fn test_authorized_responder_can_read_emergency_info() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let responder = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &30u32,
        &None,
        &PrivacyLevel::Private,
    );

    let mut allergies = Vec::new(&env);
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Penicillin"),
        severity: String::from_str(&env, "Critical"),
        is_critical: true,
    });
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Dust"),
        severity: String::from_str(&env, "Mild"),
        is_critical: false,
    });

    client.set_emergency_contacts(
        &pet_id,
        &Vec::new(&env),
        &allergies,
        &String::from_str(&env, "Needs daily medication"),
    );

    // Owner grants responder access
    client.add_emergency_responder(&pet_id, &responder);

    let info = client.get_emergency_info(&pet_id, &responder);

    // Only critical allergy returned
    assert_eq!(info.allergies.len(), 1);
    assert_eq!(
        info.allergies.get(0).unwrap().name,
        String::from_str(&env, "Penicillin")
    );
    assert!(info.allergies.get(0).unwrap().is_critical);
    assert_eq!(info.critical_alerts.len(), 1);
}

#[test]
#[should_panic(expected = "NotAuthorizedResponder")]
fn test_unauthorized_get_emergency_info_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.set_emergency_contacts(
        &pet_id,
        &Vec::new(&env),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    // Stranger not in allowlist — must panic
    client.get_emergency_info(&pet_id, &stranger);
}

#[test]
#[should_panic(expected = "NotAuthorizedResponder")]
fn test_removed_responder_cannot_read_emergency_info() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let responder = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2022-06-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "White"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.set_emergency_contacts(
        &pet_id,
        &Vec::new(&env),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    client.add_emergency_responder(&pet_id, &responder);
    client.remove_emergency_responder(&pet_id, &responder);

    // Removed responder must be rejected
    client.get_emergency_info(&pet_id, &responder);
}

#[test]
fn test_emergency_logging_records_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let responder = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.set_emergency_contacts(
        &pet_id,
        &Vec::new(&env),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    client.add_emergency_responder(&pet_id, &responder);

    client.get_emergency_info(&pet_id, &owner);
    client.get_emergency_info(&pet_id, &responder);

    let log_key = DataKey::EmergencyAccessLogs(pet_id);
    let logs: Vec<EmergencyAccessLog> = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get(&log_key)
            .unwrap_or(Vec::new(&env))
    });

    assert_eq!(logs.len(), 2);
    assert_eq!(logs.get(0).unwrap().accessed_by, owner);
    assert_eq!(logs.get(1).unwrap().accessed_by, responder);
}
