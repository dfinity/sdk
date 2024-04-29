import * as React from "react";
import { useEffect, useState } from "react";
import { Button, Col, Container, Form, Row } from "react-bootstrap"; // TODO: Import by one component.

export default function EditFoldersList(props: {
    defaultFolders?: [string, 'beginning' | 'end'][],
    defaultAntiComments?: [string, 'beginning' | 'end'][],
    onChangeFolders?: (folders: [string, 'beginning' | 'end'][]) => void,
    onChangeAntiComments?: (folders: [string, 'beginning' | 'end'][]) => void,
    noComments?: boolean,
    reverse?: boolean,
}) {
    const [folders, setFolders] = useState<[string, 'beginning' | 'end'][] | undefined>(undefined);
    const [antiComments, setAntiComments] = useState<[string, 'beginning' | 'end'][] | undefined>(undefined);
    const [side, setSide] = useState<{ [i: number]: 'beginning' | 'end' }>({});
    useEffect(() => {
        if (folders === undefined && props.defaultFolders?.length !== 0) {
            setFolders(props.defaultFolders ?? []);
        }
    }, [props.defaultFolders])
    useEffect(() => {
        if (antiComments === undefined && props.defaultAntiComments?.length !== 0) {
            setAntiComments(props.defaultAntiComments ?? []);
        }
    }, [props.defaultAntiComments])
    function updateFolders() {
        if (props.onChangeFolders !== undefined && folders !== undefined) {
            props.onChangeFolders(folders);
        }
    }
    function updateAntiComments() {
        if (props.onChangeAntiComments !== undefined && antiComments !== undefined) {
            props.onChangeAntiComments(antiComments);
        }
    }
    useEffect(updateFolders, [folders]);
    useEffect(updateAntiComments, [antiComments]);
    function updateFoldersList() {
        const list: string[] = [];
        // TODO: validation
        for (const e of document.querySelectorAll('#foldersList input[class=form-control]') as any) {
            const value = (e as HTMLInputElement).value;
            if (value !== "") {
                list.push(value)
            }
        }
        const list2: ('beginning' | 'end')[] = [];
        for (const e of document.querySelectorAll('#foldersList input[type=radio]:checked') as any) {
            const value = (e as HTMLInputElement).value;
            list2.push(value === 'beginning' ? 'beginning' : 'end');
        }
        const list3 = list.map(function(e, i) {
            const v: [string, 'beginning' | 'end'] = [e, list2[i]];
            return v;
        });
        setFolders(list3);
    }
    function updateAntiCommentsList() {
        const list: string[] = [];
        // TODO: validation
        for (const e of document.querySelectorAll('#antiCommentsList input[class=form-control]') as any) {
            const value = (e as HTMLInputElement).value;
            if (value !== "") {
                list.push(value)
            }
        }
        const list2: ('beginning' | 'end')[] = [];
        for (const e of document.querySelectorAll('#antiCommentsList input[type=radio]:checked') as any) {
            const value = (e as HTMLInputElement).value;
            list2.push(value === 'beginning' ? 'beginning' : 'end');
        }
        const list3 = list.map(function(e, i) {
            const v: [string, 'beginning' | 'end'] = [e, list2[i]];
            return v;
        });
        setAntiComments(list3);
    }
    function onSideChanged(e: React.ChangeEvent<HTMLInputElement>, i: number) {
        const newSide = {...side, [i]: ((e.currentTarget as HTMLInputElement).value === 'end' ? 'end' : 'beginning') as 'beginning' | 'end'};
        setSide(newSide);
    }

    return (
        <>
            <h2>{props.reverse ? `Folders to post` : `Post to folders`}</h2>
            <p>TODO: Visual editor of folders; TODO: Limited to ?? posts per day; TODO: begin/end works only for owned folders</p>
            <Container>
                <Row>
                    <Col>
                        <h3>Folders</h3>
                        <ul id="foldersList">
                            {(folders ?? []).map((folder, i) => {
                                return (
                                    <li key={i}>
                                        <Form.Control value={folder[0]} onChange={updateFoldersList} style={{display: 'inline', width: '15em'}}/>{" "}
                                        <Button onClick={() => setFolders(folders!.filter((item) => item !== folder))}>Delete</Button>{" "}
                                        <label><input type="radio" name={`side-f${i}`} checked={side[i] === 'beginning' || side[i] === undefined}
                                            onChange={e => onSideChanged(e, i)} value="beginning"/>&#160;beginning</label>{" "}
                                        <label><input type="radio" name={`side-f${i}`} checked={side[i] === 'end'}
                                            onChange={e => onSideChanged(e, i)} value="end"/>&#160;end</label>
                                    </li>
                                );
                            })}
                        </ul>
                        <p><Button disabled={folders === undefined} onClick={() => setFolders(folders!.concat([["", 'beginning']]))}>Add</Button></p>
                    </Col>
                    {!props.noComments &&
                    <Col>
                        <h3>Comment to</h3>
                        <ul id="antiCommentsList">
                            {(antiComments ?? []).map((folder, i) => {
                                return (
                                    <li key={i}>
                                        <Form.Control value={folder[0]} onChange={updateAntiCommentsList} style={{display: 'inline', width: '15em'}}/>{" "}
                                        <Button onClick={() => setAntiComments(antiComments!.filter((item) => item !== folder))}>Delete</Button>
                                    </li>
                                );
                            })}
                        </ul>
                        <p><Button disabled={antiComments === undefined} onClick={() => setAntiComments(antiComments!.concat([["", 'beginning']]))}>Add</Button></p>
                    </Col>}
                </Row>
            </Container>
        </>
    );
}