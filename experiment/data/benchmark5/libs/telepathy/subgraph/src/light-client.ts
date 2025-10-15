import {
  HeadUpdate as HeadUpdateEvent,
  SyncCommitteeUpdate as SyncCommitteeUpdateEvent
} from "../generated/LightClient/LightClient"
import { HeadUpdate, SyncCommitteeUpdate } from "../generated/schema"

export function handleHeadUpdate(event: HeadUpdateEvent): void {
  let entity = new HeadUpdate(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )
  entity.slot = event.params.slot
  entity.root = event.params.root

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}

export function handleSyncCommitteeUpdate(
  event: SyncCommitteeUpdateEvent
): void {
  let entity = new SyncCommitteeUpdate(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )
  entity.period = event.params.period
  entity.root = event.params.root

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}
