import React, { useContext, useRef, useState } from "react";
import { ItemData, ItemTransfer } from "../../../../declarations/CanDBPartition/CanDBPartition.did";
import { AuthContext } from "../auth/use-auth-client";
import { ItemRef, loadTotalVotes, loadUserVote, parseItemRef, serializeItemRef } from "../../data/Data";
import { idlFactory as orderIdlFactory } from "../../../../declarations/main";
import { Actor, Agent } from "@dfinity/agent";
import Button from "react-bootstrap/esm/Button";
import Modal from 'react-bootstrap/Modal';
import Nav from "react-bootstrap/esm/Nav";
import { useNavigate } from "react-router-dom";

export default function UpDown(props: {
    parent: {id: ItemRef},
    item: {order: string, id: ItemRef, item: ItemTransfer},
    agent: Agent,
    // onUpdateList: (() => void) | undefined,
    userVote: number, // -1, 0, or 1
    onSetUserVote: (id: ItemRef, v: number) => void,
    totalVotes: { up: number, down: number },
    onSetTotalVotes: (id: ItemRef, v: { up: number, down: number }) => void,
    onUpdateList: (() => void) | undefined,
    isComment?: boolean,
}) {
    const { principal, agent } = useContext(AuthContext) as any;
    // const dialogRef = useRef();
    const [showDialog, setShowDialog] = useState(false);

    // hack
    async function vote(value: number, clicked: 'up' | 'down', isComment) {
        if (principal === undefined || principal.toString() === "2vxsx-fae") { // TODO: hack
            alert("Login to vote!"); // TODO: a better dialog
            return;
        }

        let changeUp = (value == 1 && props.userVote != 1) || (props.userVote == 1 && value != 1);
        let changeDown = (value == -1 && props.userVote != -1) || (props.userVote == -1 && value != -1);

        let up = props.totalVotes ? Number(props.totalVotes.up) : 0;
        let down = props.totalVotes ? Number(props.totalVotes.down) : 0;
        if (changeUp || changeDown) {
            if (changeUp) {
              up += value - Number(props.userVote) > 0 ? 1 : -1;
            }
            if (changeDown) {
              down += value - Number(props.userVote) > 0 ? -1 : 1;
            }
        }      

        const order = Actor.createActor(orderIdlFactory, {canisterId: process.env.CANISTER_ID_ORDER!, agent});
        try {
            await order.vote(
                props.parent.id.canister,
                BigInt(props.parent.id.id),
                props.item.id.canister,
                BigInt(props.item.id.id),
                BigInt(value),
                isComment === true
            );
            if (clicked === 'up') {
                props.onSetUserVote(props.item.id, props.userVote === 1 ? 0 : 1);
            };
            if (clicked === 'down') {
                props.onSetUserVote(props.item.id, props.userVote === -1 ? 0 : -1);
            };
            props.onSetTotalVotes(props.item.id, {up, down});
            if (props.onUpdateList !== undefined) {
                props.onUpdateList();
            }
        }
        catch (e) { // TODO: more specific event
            console.log("VOTE", e);
            setShowDialog(true);
        }
    }
    function votesTitle() {
        return props.totalVotes ? `Up: ${props.totalVotes.up} Down: ${props.totalVotes.down}` : "";
    }

    const navigate = useNavigate();
    // TODO: Is it OK to have a separate modal for each item?
    return (
        <span title={votesTitle()}>
            <Modal show={showDialog} onHide={() => setShowDialog(false)}>
                <Modal.Dialog>
                    <Modal.Header closeButton>
                        <Modal.Title>Need to authorize</Modal.Title>
                    </Modal.Header>

                    <Modal.Body>
                    <p>Confirm that you are a real person:</p>
                    <p>
                        <Nav.Link onClick={() => navigate("/personhood")} style={{color: 'blue'}}>
                            Verify Your Account
                        </Nav.Link>
                    </p>
                    </Modal.Body>
                    <Modal.Footer>
                        <Button onClick={() => setShowDialog(false)} variant="secondary">Close</Button>
                    </Modal.Footer>
                </Modal.Dialog>
            </Modal>
            <Button
                onClick={async e => await vote((e.target as Element).classList.contains('active') ? 0 : +1, 'up', props.isComment === true)}
                className={props.userVote > 0 ? 'thumbs active' : 'thumbs'}>👍</Button>
            <Button
                onClick={async e => await vote((e.target as Element).classList.contains('active') ? 0 : -1, 'down', props.isComment === true)}
                className={props.userVote < 0 ? 'thumbs active' : 'thumbs'}>👎</Button>
        </span>
    );
}

export async function updateVotes(agent: Agent, id, principal, source: {order: string, id: ItemRef, item: ItemTransfer}[], setTotalVotes, setUserVote) { // TODO: argument types
    const totalVotes: {[key: string]: {up: number, down: number}} = {};
    const totalVotesPromises = (source || []).map(folder =>
        loadTotalVotes(agent, id!, folder.id).then(res => {
            totalVotes[serializeItemRef(folder.id)] = res;
        }),
    );
    Promise.all(totalVotesPromises).then(() => {
        // TODO: Remove votes for excluded items?
        setTotalVotes(totalVotes); // TODO: Set it instead above in the loop for faster results?
    });

    if (principal) {
        const userVotes: {[key: string]: number} = {};
        const userVotesPromises = (source || []).map(folder =>
            loadUserVote(agent, principal, id!, folder.id).then(res => {
                userVotes[serializeItemRef(folder.id)] = res;
            }),
        );
        Promise.all(userVotesPromises).then(() => {
            setUserVote(userVotes); // TODO: Set it instead above in the loop for faster results?
        });
    }
}
