// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// State module represents the current state of the pool. It maintains all
/// the accounts, history, and governance information. It also processes all
/// the transactions and updates the state accordingly.
module deepbook::state;

use deepbook::account::{Self, Account};
use deepbook::balances::{Self, Balances};
use deepbook::constants;
use deepbook::fill::Fill;
use deepbook::governance::{Self, Governance};
use deepbook::history::{Self, History};
use deepbook::math;
use deepbook::order::Order;
use deepbook::order_info::OrderInfo;
use sui::event;
use sui::table::{Self, Table};

// === Errors ===
const ENoStake: u64 = 1;
const EMaxOpenOrders: u64 = 2;
const EAlreadyProposed: u64 = 3;

// === Structs ===
public struct State has store {
    accounts: Table<ID, Account>,
    history: History,
    governance: Governance,
}

public struct StakeEvent has copy, drop {
    pool_id: ID,
    balance_manager_id: ID,
    epoch: u64,
    amount: u64,
    stake: bool,
}

public struct ProposalEvent has copy, drop {
    pool_id: ID,
    balance_manager_id: ID,
    epoch: u64,
    taker_fee: u64,
    maker_fee: u64,
    stake_required: u64,
}

public struct VoteEvent has copy, drop {
    pool_id: ID,
    balance_manager_id: ID,
    epoch: u64,
    from_proposal_id: Option<ID>,
    to_proposal_id: ID,
    stake: u64,
}

public struct RebateEvent has copy, drop {
    pool_id: ID,
    balance_manager_id: ID,
    epoch: u64,
    claim_amount: u64,
}

public(package) fun empty(stable_pool: bool, ctx: &mut TxContext): State {
    let governance = governance::empty(
        stable_pool,
        ctx,
    );
    let trade_params = governance.trade_params();
    let history = history::empty(trade_params, ctx.epoch(), ctx);

    State { history, governance, accounts: table::new(ctx) }
}

/// Up until this point, an OrderInfo object has been created and potentially filled.
/// The OrderInfo object contains all of the necessary information to update the state
/// of the pool. This includes the volumes for the taker and potentially multiple makers.
/// First, fills are iterated and processed, updating the appropriate user's volumes.
/// Funds are settled for those makers. Then, the taker's trading fee is calculated
/// and the taker's volumes are updated. Finally, the taker's balances are settled.
public(package) fun process_create(
    self: &mut State,
    order_info: &mut OrderInfo,
    ctx: &TxContext,
): (Balances, Balances) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    let fills = order_info.fills_ref();
    self.process_fills(fills, ctx);

    self.update_account(order_info.balance_manager_id(), ctx);
    let account = &mut self.accounts[order_info.balance_manager_id()];
    let account_volume = account.total_volume();
    let account_stake = account.active_stake();

    // avg exucuted price for taker
    let avg_executed_price = if (order_info.executed_quantity() > 0) {
        math::div(
            order_info.cumulative_quote_quantity(),
            order_info.executed_quantity(),
        )
    } else {
        0
    };
    let account_volume_in_deep = order_info
        .order_deep_price()
        .deep_quantity_u128(
            account_volume,
            math::mul_u128(account_volume, avg_executed_price as u128),
        );

    // taker fee will almost be calculated as 0 for whitelisted pools by default, as account_volume_in_deep is 0
    let taker_fee = self
        .governance
        .trade_params()
        .taker_fee_for_user(account_stake, account_volume_in_deep);
    let maker_fee = self.governance.trade_params().maker_fee();

    if (order_info.order_inserted()) {
        assert!(
            account.open_orders().size() < constants::max_open_orders(),
            EMaxOpenOrders,
        );
        account.add_order(order_info.order_id());
    };
    account.add_taker_volume(order_info.executed_quantity());

    let (mut settled, mut owed) = order_info.calculate_partial_fill_balances(
        taker_fee,
        maker_fee,
    );
    let (old_settled, old_owed) = account.settle();
    self
        .history
        .add_total_fees_collected(balances::new(0, 0, order_info.paid_fees()));
    settled.add_balances(old_settled);
    owed.add_balances(old_owed);

    (settled, owed)
}

public(package) fun withdraw_settled_amounts(
    self: &mut State,
    balance_manager_id: ID,
): (Balances, Balances) {
    if (self.accounts.contains(balance_manager_id)) {
        let account = &mut self.accounts[balance_manager_id];

        account.settle()    
    } else {
        (balances::empty(), balances::empty())
    }
    
}

/// Update account settled balances and volumes.
/// Remove order from account orders.
public(package) fun process_cancel(
    self: &mut State,
    order: &mut Order,
    balance_manager_id: ID,
    ctx: &TxContext,
): (Balances, Balances) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);
    order.set_canceled();

    let epoch = order.epoch();
    let maker_fee = self.history.historic_maker_fee(epoch);
    let balances = order.calculate_cancel_refund(maker_fee, option::none());

    let account = &mut self.accounts[balance_manager_id];
    account.remove_order(order.order_id());
    account.add_settled_balances(balances);

    account.settle()
}

/// Given the modified quantity, update account settled balances and volumes.
public(package) fun process_modify(
    self: &mut State,
    balance_manager_id: ID,
    cancel_quantity: u64,
    order: &Order,
    ctx: &TxContext,
): (Balances, Balances) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);

    let epoch = order.epoch();
    let maker_fee = self.history.historic_maker_fee(epoch);
    let balances = order.calculate_cancel_refund(
        maker_fee,
        option::some(cancel_quantity),
    );

    self.accounts[balance_manager_id].add_settled_balances(balances);

    self.accounts[balance_manager_id].settle()
}

/// Process stake transaction. Add stake to account and update governance.
public(package) fun process_stake(
    self: &mut State,
    pool_id: ID,
    balance_manager_id: ID,
    new_stake: u64,
    ctx: &TxContext,
): (Balances, Balances) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);

    let (stake_before, stake_after) = self
        .accounts[balance_manager_id]
        .add_stake(new_stake);
    self.governance.adjust_voting_power(stake_before, stake_after);
    event::emit(StakeEvent {
        pool_id,
        balance_manager_id,
        epoch: ctx.epoch(),
        amount: new_stake,
        stake: true,
    });

    self.accounts[balance_manager_id].settle()
}

/// Process unstake transaction. Remove stake from account and update governance.
public(package) fun process_unstake(
    self: &mut State,
    pool_id: ID,
    balance_manager_id: ID,
    ctx: &TxContext,
): (Balances, Balances) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);

    let account = &mut self.accounts[balance_manager_id];
    let active_stake = account.active_stake();
    let inactive_stake = account.inactive_stake();
    let voted_proposal = account.voted_proposal();
    account.remove_stake();
    self.governance.adjust_voting_power(active_stake + inactive_stake, 0);
    self.governance.adjust_vote(voted_proposal, option::none(), active_stake);
    event::emit(StakeEvent {
        pool_id,
        balance_manager_id,
        epoch: ctx.epoch(),
        amount: active_stake + inactive_stake,
        stake: false,
    });

    account.settle()
}

/// Process proposal transaction. Add proposal to governance and update account.
public(package) fun process_proposal(
    self: &mut State,
    pool_id: ID,
    balance_manager_id: ID,
    taker_fee: u64,
    maker_fee: u64,
    stake_required: u64,
    ctx: &TxContext,
) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);
    let account = &mut self.accounts[balance_manager_id];
    let stake = account.active_stake();
    let proposal_created = account.created_proposal();

    assert!(stake > 0, ENoStake);
    assert!(!proposal_created, EAlreadyProposed);
    account.set_created_proposal(true);

    self
        .governance
        .add_proposal(
            taker_fee,
            maker_fee,
            stake_required,
            stake,
            balance_manager_id,
        );
    self.process_vote(pool_id, balance_manager_id, balance_manager_id, ctx);

    event::emit(ProposalEvent {
        pool_id,
        balance_manager_id,
        epoch: ctx.epoch(),
        taker_fee,
        maker_fee,
        stake_required,
    });
}

/// Process vote transaction. Update account voted proposal and governance.
public(package) fun process_vote(
    self: &mut State,
    pool_id: ID,
    balance_manager_id: ID,
    proposal_id: ID,
    ctx: &TxContext,
) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);

    let account = &mut self.accounts[balance_manager_id];
    assert!(account.active_stake() > 0, ENoStake);

    let prev_proposal = account.set_voted_proposal(option::some(proposal_id));
    self
        .governance
        .adjust_vote(
            prev_proposal,
            option::some(proposal_id),
            account.active_stake(),
        );

    event::emit(VoteEvent {
        pool_id,
        balance_manager_id,
        epoch: ctx.epoch(),
        from_proposal_id: prev_proposal,
        to_proposal_id: proposal_id,
        stake: account.active_stake(),
    });
}

/// Process claim rebates transaction. Update account rebates and settle balances.
public(package) fun process_claim_rebates(
    self: &mut State,
    pool_id: ID,
    balance_manager_id: ID,
    ctx: &TxContext,
): (Balances, Balances) {
    self.governance.update(ctx);
    self.history.update(self.governance.trade_params(), ctx);
    self.update_account(balance_manager_id, ctx);

    let account = &mut self.accounts[balance_manager_id];
    let claim_amount = account.claim_rebates();
    event::emit(RebateEvent {
        pool_id,
        balance_manager_id,
        epoch: ctx.epoch(),
        claim_amount,
    });

    account.settle()
}

public(package) fun governance(self: &State): &Governance {
    &self.governance
}

public(package) fun governance_mut(
    self: &mut State,
    ctx: &TxContext,
): &mut Governance {
    self.governance.update(ctx);

    &mut self.governance
}

public(package) fun account_exists(self: &State, balance_manager_id: ID): bool {
    self.accounts.contains(balance_manager_id)
}

public(package) fun account(self: &State, balance_manager_id: ID): &Account {
    &self.accounts[balance_manager_id]
}

public(package) fun history_mut(self: &mut State): &mut History {
    &mut self.history
}

public(package) fun history(self: &State): &History {
    &self.history
}

// === Private Functions ===
/// Process fills for all makers. Update maker accounts and history.
fun process_fills(self: &mut State, fills: &mut vector<Fill>, ctx: &TxContext) {
    let whitelisted = self.governance.whitelisted();

    let mut i = 0;
    while (i < fills.length()) {
        let fill = &mut fills[i];
        let maker = fill.balance_manager_id();
        self.update_account(maker, ctx);
        let account = &mut self.accounts[maker];
        account.process_maker_fill(fill);

        let base_volume = fill.base_quantity();
        let quote_volume = fill.quote_quantity();
        let historic_maker_fee = self
            .history
            .historic_maker_fee(fill.maker_epoch());
        let fee_volume = fill
            .maker_deep_price()
            .deep_quantity(base_volume, quote_volume);
        let order_maker_fee = if (whitelisted) {
            0
        } else {
            math::mul(fee_volume, historic_maker_fee)
        };

        if (!fill.expired()) {
            if (order_maker_fee > 0) {
                fill.set_fill_maker_fee(order_maker_fee);
            };
            self.history.add_volume(base_volume, account.active_stake());
            self
                .history
                .add_total_fees_collected(balances::new(0, 0, order_maker_fee));
        } else {
            account.add_settled_balances(balances::new(0, 0, order_maker_fee));
        };

        i = i + 1;
    };
}

/// If account doesn't exist, create it. Update account volumes and rebates.
fun update_account(self: &mut State, balance_manager_id: ID, ctx: &TxContext) {
    if (!self.accounts.contains(balance_manager_id)) {
        self.accounts.add(balance_manager_id, account::empty(ctx));
    };

    let account = &mut self.accounts[balance_manager_id];
    let (prev_epoch, maker_volume, active_stake) = account.update(ctx);
    if (prev_epoch > 0 && maker_volume > 0 && active_stake > 0) {
        let rebates = self
            .history
            .calculate_rebate_amount(prev_epoch, maker_volume, active_stake);
        account.add_rebates(rebates);
    }
}