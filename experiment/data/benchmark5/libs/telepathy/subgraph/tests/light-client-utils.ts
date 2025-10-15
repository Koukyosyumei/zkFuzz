import { newMockEvent } from "matchstick-as"
import { ethereum, BigInt, Bytes } from "@graphprotocol/graph-ts"
import {
  HeadUpdate,
  SyncCommitteeUpdate
} from "../generated/LightClient/LightClient"

export function createHeadUpdateEvent(slot: BigInt, root: Bytes): HeadUpdate {
  let headUpdateEvent = changetype<HeadUpdate>(newMockEvent())

  headUpdateEvent.parameters = new Array()

  headUpdateEvent.parameters.push(
    new ethereum.EventParam("slot", ethereum.Value.fromUnsignedBigInt(slot))
  )
  headUpdateEvent.parameters.push(
    new ethereum.EventParam("root", ethereum.Value.fromFixedBytes(root))
  )

  return headUpdateEvent
}

export function createSyncCommitteeUpdateEvent(
  period: BigInt,
  root: Bytes
): SyncCommitteeUpdate {
  let syncCommitteeUpdateEvent = changetype<SyncCommitteeUpdate>(newMockEvent())

  syncCommitteeUpdateEvent.parameters = new Array()

  syncCommitteeUpdateEvent.parameters.push(
    new ethereum.EventParam("period", ethereum.Value.fromUnsignedBigInt(period))
  )
  syncCommitteeUpdateEvent.parameters.push(
    new ethereum.EventParam("root", ethereum.Value.fromFixedBytes(root))
  )

  return syncCommitteeUpdateEvent
}
