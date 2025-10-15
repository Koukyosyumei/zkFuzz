import {
  assert,
  describe,
  test,
  clearStore,
  beforeAll,
  afterAll
} from "matchstick-as/assembly/index"
import { BigInt, Bytes } from "@graphprotocol/graph-ts"
import { HeadUpdate } from "../generated/schema"
import { HeadUpdate as HeadUpdateEvent } from "../generated/LightClient/LightClient"
import { handleHeadUpdate } from "../src/light-client"
import { createHeadUpdateEvent } from "./light-client-utils"

// Tests structure (matchstick-as >=0.5.0)
// https://thegraph.com/docs/en/developer/matchstick/#tests-structure-0-5-0

describe("Describe entity assertions", () => {
  beforeAll(() => {
    let slot = BigInt.fromI32(234)
    let root = Bytes.fromI32(1234567890)
    let newHeadUpdateEvent = createHeadUpdateEvent(slot, root)
    handleHeadUpdate(newHeadUpdateEvent)
  })

  afterAll(() => {
    clearStore()
  })

  // For more test scenarios, see:
  // https://thegraph.com/docs/en/developer/matchstick/#write-a-unit-test

  test("HeadUpdate created and stored", () => {
    assert.entityCount("HeadUpdate", 1)

    // 0xa16081f360e3847006db660bae1c6d1b2e17ec2a is the default address used in newMockEvent() function
    assert.fieldEquals(
      "HeadUpdate",
      "0xa16081f360e3847006db660bae1c6d1b2e17ec2a-1",
      "slot",
      "234"
    )
    assert.fieldEquals(
      "HeadUpdate",
      "0xa16081f360e3847006db660bae1c6d1b2e17ec2a-1",
      "root",
      "1234567890"
    )

    // More assert options:
    // https://thegraph.com/docs/en/developer/matchstick/#asserts
  })
})
