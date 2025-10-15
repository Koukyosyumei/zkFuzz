pragma circom 2.0.0;

include "../libs/hydra-2010a65/hydra-s1.circom";

component main {public [commitmentMapperPubKey, registryTreeRoot, ticketIdentifier, userTicket, destinationIdentifier, claimedValue, chainId, accountsTreeValue, isStrict]} = hydraS1(20,20);
